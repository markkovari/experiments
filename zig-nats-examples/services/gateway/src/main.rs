use axum::{
    body::{Body},
    extract::{Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use config::AppConfig;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    nats_client: async_nats::Client,
    kv: async_nats::jetstream::kv::Store,
    http_client: reqwest::Client,
    config: AppConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "gateway=debug,info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting API Gateway...");

    let settings = AppConfig::load().map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    let nats_client = async_nats::connect(&settings.nats.url).await?;
    
    // Setup NATS KV for rate limiting
    let js = async_nats::jetstream::new(nats_client.clone());
    let kv = js.create_key_value(async_nats::jetstream::kv::Config {
        bucket: "RATE_LIMITS".to_string(),
        history: 1,
        ..Default::default()
    }).await?;

    let state = AppState {
        nats_client,
        kv,
        http_client: reqwest::Client::new(),
        config: settings.clone(),
    };

    let app = Router::new()
        .route("/api/*path", any(proxy_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", settings.server.host, settings.server.port).parse()?;
    tracing::info!("Gateway listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn proxy_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request,
) -> Response {
    // 1. Rate Limiting Logic
    let identifier = req.headers().get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let kv_key = format!("rate_limit:{}", identifier);
    
    let current_count = match state.kv.get(&kv_key).await {
        Ok(Some(entry)) => {
            let val = String::from_utf8_lossy(&entry).parse::<u32>().unwrap_or(0);
            val
        }
        _ => 0,
    };

    if current_count > 1000 {
        return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
    }

    let _ = state.kv.put(&kv_key, (current_count + 1).to_string().into()).await;

    // 2. Routing Logic
    let internal_url = if path.starts_with("auth") {
        format!("http://localhost:3000{}", &path[4..])
    } else if path.starts_with("orgs") {
        format!("http://localhost:3001/orgs{}", &path[4..])
    } else if path.starts_with("actions") {
        format!("http://localhost:3002/actions{}", &path[7..])
    } else if path.starts_with("executions") {
        format!("http://localhost:3002/executions{}", &path[10..])
    } else {
        return (StatusCode::NOT_FOUND, "Service not found").into_response();
    };

    // 3. Forward Request
    let method = req.method().clone();
    let headers = req.headers().clone();
    
    let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response(),
    };

    let mut proxy_req = state.http_client.request(method, internal_url);
    for (key, value) in headers.iter() {
        if key != "host" {
            proxy_req = proxy_req.header(key, value);
        }
    }

    let res = match proxy_req.body(body_bytes).send().await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Proxy error: {}", e);
            return (StatusCode::BAD_GATEWAY, "Internal service unavailable").into_response();
        }
    };

    // 4. Return Response
    let status = res.status();
    let res_headers = res.headers().clone();
    let res_body_bytes = match res.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read internal response").into_response(),
    };

    let mut response = Response::new(Body::from(res_body_bytes));
    *response.status_mut() = status;
    for (key, value) in res_headers.iter() {
        response.headers_mut().insert(key, value.clone());
    }

    response
}
