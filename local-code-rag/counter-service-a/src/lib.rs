use wasmcloud_component::http;
use wasmcloud_component::http::ErrorCode;
use wasmcloud_component::wasi::keyvalue::*;

struct Counter;

http::export!(Counter);

impl http::Server for Counter {
    fn handle(
        request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        let (parts, _) = request.into_parts();

        let Some(path_with_query) = parts.uri.path_and_query() else {
            return http::Response::builder()
                .status(400)
                .body("Bad request: missing path".into())
                .map_err(|e| {
                    ErrorCode::InternalError(Some(format!("response build error: {e:?}")))
                });
        };

        let key = path_with_query.path().trim_start_matches('/');

        let bucket = store::open("default")
            .map_err(|e| ErrorCode::InternalError(Some(format!("bucket open failed: {e:?}"))))?;

        let count = atomics::increment(&bucket, key, 1)
            .map_err(|e| ErrorCode::InternalError(Some(format!("increment failed: {e:?}"))))?;

        Ok(http::Response::new(format!(
            "Counter '{key}': {count}\n"
        )))
    }
}
