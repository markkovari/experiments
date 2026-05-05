use axum::{
    body::{Body, Bytes},
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

    tracing::info!("Starting API Gateway (v6)...");

    let settings = AppConfig::load().map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    
    // Connect to NATS with retry
    let nats_client = {
        let mut retry_count = 0;
        loop {
            match async_nats::connect(&settings.nats.url).await {
                Ok(client) => break client,
                Err(e) if retry_count < 30 => {
                    retry_count += 1;
                    tracing::warn!("Failed to connect to NATS at {}, retrying ({}/30): {}", settings.nats.url, retry_count, e);
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
                Err(e) => return Err(anyhow::anyhow!("Final NATS connection failure: {}", e)),
            }
        }
    };
    tracing::info!("Connected to NATS at {}", settings.nats.url);
    
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
    // 1. Rate Limiting
    let identifier = req.headers().get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let kv_key = format!("rate_limit:{}", identifier.replace(".", "_").replace(":", "_"));
    
    let current_count = match state.kv.get(&kv_key).await {
        Ok(Some(value)) => {
            String::from_utf8_lossy(&value).parse::<u32>().unwrap_or(0)
        }
        _ => 0,
    };

    if current_count > 5000 {
        return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
    }

    let _ = state.kv.put(&kv_key, (current_count + 1).to_string().into()).await;

    let query_string = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();

    // 2. Routing
    let mut internal_url = if path.starts_with("auth") {
        // Auth service expects /login, /register (strip "auth")
        let base = std::env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        format!("{}{}", base, &path[4..])
    } else if path.starts_with("orgs") {
        // Org service expects /orgs (DO NOT STRIP)
        let base = std::env::var("ORG_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3001/orgs".to_string());
        format!("{}/{}", base.trim_end_matches('/'), path)
    } else if path.starts_with("actions") || path.starts_with("executions") {
        // Scheduler expects /actions, /executions (DO NOT STRIP)
        let base = std::env::var("SCHEDULER_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());
        format!("{}/{}", base.trim_end_matches('/'), path)
    } else {
        return (StatusCode::NOT_FOUND, "Service not found").into_response();
    };

    internal_url.push_str(&query_string);

    tracing::info!("PROXY: {} /api/{} -> {}", req.method(), path, internal_url);

    // 3. Forward
    let method = req.method().clone();
    let headers = req.headers().clone();
    
    let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read request body: {}", e);
            return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response();
        }
    };

    let mut proxy_req = state.http_client.request(method, &internal_url);
    for (key, value) in headers.iter() {
        if key != "host" {
            proxy_req = proxy_req.header(key, value);
        }
    }

    let res = match proxy_req.body(body_bytes).send().await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Proxy error for {}: {}", path, e);
            return (StatusCode::BAD_GATEWAY, format!("Internal service unavailable: {}", e)).into_response();
        }
    };

    // 4. Return
    let status = res.status();
    let res_headers = res.headers().clone();
    
    tracing::info!("PROXY RESPONSE: {} -> {}", internal_url, status);

    let res_body_bytes = match res.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read internal response from {}: {}", internal_url, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read internal response").into_response();
        }
    };

    let mut response = Response::new(Body::from(res_body_bytes));
    *response.status_mut() = status;
    for (key, value) in res_headers.iter() {
        response.headers_mut().insert(key, value.clone());
    }

    response
}
