use wasmcloud_component::http;
use wasmcloud_component::http::ErrorCode;
use wasmcloud_component::wasi::keyvalue::*;
use wasmcloud_component::wasmcloud::bus::lattice;

struct Component;

http::export!(Component);

impl http::Server for Component {
    fn handle(
        request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        let (parts, _) = request.into_parts();

        let Some(path_with_query) = parts.uri.path_and_query() else {
            return http::Response::builder()
                .status(400)
                .body("Bad request, did not contain path and query".to_string())
                .map_err(|e| {
                    ErrorCode::InternalError(Some(format!("failed to build response {e:?}")))
                });
        };

        let object_name = path_with_query.path();
        let method = parts.method;

        // Set the link name before performing keyvalue operations
        lattice::set_link_name(
            "default",
            vec![
                lattice::CallTargetInterface::new("wasi", "keyvalue", "store"),
                lattice::CallTargetInterface::new("wasi", "keyvalue", "atomics"),
            ],
        )
        .map_err(|e| ErrorCode::InternalError(Some(format!("failed to set link name {e:?}"))))?;

        // Open the KV bucket
        let bucket = store::open("default")
            .map_err(|e| ErrorCode::InternalError(Some(format!("failed to open bucket: {e:?}"))))?;

        let trimmed_path = object_name.trim_start_matches('/');

        // Handle GET / - return info message
        if trimmed_path.is_empty() && method == http::Method::GET {
            return http::Response::builder()
                .status(200)
                .body("{\"message\": \"Counter service. Use POST /:name to increment, GET /:name to read.\"}\n".to_string())
                .map_err(|e| ErrorCode::InternalError(Some(format!("{e:?}"))));
        }

        // Reject empty paths
        if trimmed_path.is_empty() {
            return http::Response::builder()
                .status(404)
                .body("{\"error\": \"Not found\"}\n".to_string())
                .map_err(|e| ErrorCode::InternalError(Some(format!("{e:?}"))));
        }

        // Handle GET /:name - read counter by incrementing by 0
        if method == http::Method::GET {
            return match atomics::increment(&bucket, trimmed_path, 0) {
                Ok(value) => http::Response::builder()
                    .status(200)
                    .body(format!("{{\"name\": \"{}\", \"value\": {}}}\n", trimmed_path, value))
                    .map_err(|e| ErrorCode::InternalError(Some(format!("{e:?}")))),
                Err(_) => http::Response::builder()
                    .status(404)
                    .body("{\"error\": \"Counter not found\"}\n".to_string())
                    .map_err(|e| ErrorCode::InternalError(Some(format!("{e:?}")))),
            };
        }

        // Handle POST /:name - increment counter
        if method != http::Method::POST {
            return http::Response::builder()
                .status(405)
                .body("{\"error\": \"Method not allowed\"}\n".to_string())
                .map_err(|e| ErrorCode::InternalError(Some(format!("{e:?}"))));
        }

        let count = atomics::increment(&bucket, trimmed_path, 1).map_err(|e| {
            ErrorCode::InternalError(Some(format!("failed to increment counter {e:?}")))
        })?;

        http::Response::builder()
            .status(200)
            .body(format!("{{\"name\": \"{}\", \"value\": {}}}\n", trimmed_path, count))
            .map_err(|e| ErrorCode::InternalError(Some(format!("{e:?}"))))
    }
}
