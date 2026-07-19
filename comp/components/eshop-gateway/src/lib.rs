//! eshop:gateway — the Envoy + Blazor-host stand-in: embedded storefront SPA
//! plus a reverse proxy to the sibling services. Base URLs are wasi:config so
//! the same wasm fronts localhost (native lane) and cluster DNS (k8s).
//!
//! POST /internal/pump fans out to every consumer service's pump, so one
//! driver (the pump loop, or the SPA itself while open) advances the whole
//! choreography.

#[allow(warnings)]
mod bindings;

use bindings::wasi::config::store as config;
use bindings::wasi::http::outgoing_handler;
use bindings::wasi::http::types::{
    Fields, IncomingRequest, Method, OutgoingBody, OutgoingRequest, OutgoingResponse,
    RequestOptions, ResponseOutparam, Scheme,
};
use bindings::wasi::io::streams::StreamError;

// ponytail: single-file SPA (the jco-helpdesk pattern) include_str!'d here;
// switch to the static-assets component if the UI ever needs a build step.
const INDEX_HTML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/eshop/ui/index.html"));

struct Component;

fn base(key: &str, default_port: u16) -> String {
    config::get(key)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("http://127.0.0.1:{default_port}"))
}

impl bindings::exports::wasi::http::incoming_handler::Guest for Component {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        let method = request.method();
        let path = request.path_with_query().unwrap_or_else(|| "/".to_string());
        let route = path.split('?').next().unwrap_or("/").to_string();
        let seg: Vec<&str> = route.trim_matches('/').split('/').collect();

        let identity = base("eshop-identity-url", 3105);
        let catalog = base("eshop-catalog-url", 3101);
        let basket = base("eshop-basket-url", 3102);
        let ordering = base("eshop-ordering-url", 3103);
        let payment = base("eshop-payment-url", 3104);

        match seg.as_slice() {
            // identity keeps its own root-level routes; strip the prefix.
            ["api", "identity", rest @ ..] => {
                let target = format!("{identity}/{}", rest.join("/"));
                proxy(&request, &method, &target, response_out);
            }
            ["api", "catalog", ..] => proxy(&request, &method, &format!("{catalog}{path}"), response_out),
            ["api", "basket", ..] => proxy(&request, &method, &format!("{basket}{path}"), response_out),
            ["api", "orders", ..] => proxy(&request, &method, &format!("{ordering}{path}"), response_out),
            ["internal", "pump"] => {
                // Ordering first (creates/advances), then the reactors.
                let mut ok = 0;
                for svc in [&ordering, &catalog, &payment, &basket] {
                    if fetch(&Method::Post, &format!("{svc}/internal/pump"), None, &[]).is_some() {
                        ok += 1;
                    }
                }
                let body = format!("{{\"pumped\":{ok}}}");
                respond(response_out, 200, "application/json", body.as_bytes());
            }
            // everything else is the storefront (SPA fallback included).
            _ => respond(response_out, 200, "text/html; charset=utf-8", INDEX_HTML.as_bytes()),
        }
    }
}

/// Forward the incoming request to `target`, mirroring the upstream answer.
fn proxy(request: &IncomingRequest, method: &Method, target: &str, response_out: ResponseOutparam) {
    let auth = request
        .headers()
        .get(&"authorization".to_string())
        .into_iter()
        .next()
        .and_then(|v| String::from_utf8(v).ok());
    let body = match method {
        Method::Get | Method::Head => Vec::new(),
        _ => read_body(request),
    };
    let mut extra: Vec<(&str, String)> = Vec::new();
    if let Some(a) = auth {
        extra.push(("authorization", a));
    }
    match fetch(method, target, Some(&body), &extra) {
        Some((status, ct, bytes)) => respond(response_out, status, &ct, &bytes),
        None => respond(
            response_out,
            502,
            "application/json",
            format!("{{\"error\":\"upstream unreachable: {target}\"}}").as_bytes(),
        ),
    }
}

/// One outgoing HTTP round trip. Returns (status, content-type, body).
fn fetch(
    method: &Method,
    url: &str,
    body: Option<&[u8]>,
    extra_headers: &[(&str, String)],
) -> Option<(u16, String, Vec<u8>)> {
    let (scheme, rest) = if let Some(r) = url.strip_prefix("https://") {
        (Scheme::Https, r)
    } else {
        (Scheme::Http, url.strip_prefix("http://")?)
    };
    let (authority, path) = match rest.find('/') {
        Some(i) => (rest[..i].to_string(), rest[i..].to_string()),
        None => (rest.to_string(), "/".to_string()),
    };

    let headers = Fields::new();
    let _ = headers.set(&"content-type".to_string(), &[b"application/json".to_vec()]);
    for (k, v) in extra_headers {
        let _ = headers.set(&k.to_string(), &[v.as_bytes().to_vec()]);
    }
    let req = OutgoingRequest::new(headers);
    req.set_method(method).ok()?;
    req.set_scheme(Some(&scheme)).ok()?;
    req.set_authority(Some(&authority)).ok()?;
    req.set_path_with_query(Some(&path)).ok()?;
    {
        let out = req.body().ok()?;
        if let Some(bytes) = body {
            if !bytes.is_empty() {
                let stream = out.write().ok()?;
                for chunk in bytes.chunks(4096) {
                    stream.blocking_write_and_flush(chunk).ok()?;
                }
            }
        }
        OutgoingBody::finish(out, None).ok()?;
    }

    let future = outgoing_handler::handle(req, Some(RequestOptions::new())).ok()?;
    future.subscribe().block();
    let resp = future.get()?.ok()?.ok()?;
    let status = resp.status();
    let ct = resp
        .headers()
        .get(&"content-type".to_string())
        .into_iter()
        .next()
        .and_then(|v| String::from_utf8(v).ok())
        .unwrap_or_else(|| "application/json".into());
    let mut bytes = Vec::new();
    if let Ok(incoming) = resp.consume() {
        if let Ok(stream) = incoming.stream() {
            loop {
                match stream.blocking_read(8192) {
                    Ok(c) if c.is_empty() => break,
                    Ok(c) => bytes.extend_from_slice(&c),
                    Err(StreamError::Closed) => break,
                    Err(_) => break,
                }
            }
        }
    }
    Some((status, ct, bytes))
}

fn read_body(request: &IncomingRequest) -> Vec<u8> {
    let mut buf = Vec::new();
    if let Ok(body) = request.consume() {
        if let Ok(stream) = body.stream() {
            loop {
                match stream.blocking_read(8192) {
                    Ok(c) if c.is_empty() => break,
                    Ok(c) => buf.extend_from_slice(&c),
                    Err(_) => break,
                }
            }
        }
    }
    buf
}

fn respond(response_out: ResponseOutparam, status: u16, content_type: &str, body: &[u8]) {
    let headers = Fields::new();
    let _ = headers.set(&"content-type".to_string(), &[content_type.as_bytes().to_vec()]);
    let response = OutgoingResponse::new(headers);
    let _ = response.set_status_code(status);
    let out = response.body().expect("outgoing body");
    ResponseOutparam::set(response_out, Ok(response));
    if !body.is_empty() {
        let stream = out.write().expect("write stream");
        for chunk in body.chunks(4096) {
            let _ = stream.blocking_write_and_flush(chunk);
        }
    }
    let _ = OutgoingBody::finish(out, None);
}

bindings::export!(Component with_types_in bindings);
