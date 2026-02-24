use serde::Serialize;
use wasmcloud_component::http;
use wasmcloud_component::http::ErrorCode;
use wasmcloud_component::wasi::keyvalue::*;
use wasmcloud_component::wasmcloud::bus::lattice;

#[derive(Serialize)]
struct CounterData {
    name: String,
    value: u64,
}

#[derive(Serialize)]
struct InfoMessage {
    message: String,
}

#[derive(Serialize)]
struct ErrorMessage {
    error: String,
}

struct Component;

http::export!(Component);

/// Helper function to create a JSON response with proper error handling
fn json_response<T: Serialize>(data: &T, status: u16) -> http::Result<http::Response<String>> {
    let mut json = serde_json::to_string(data)
        .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
    json.push('\n');

    http::Response::builder()
        .status(status)
        .body(json)
        .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))
}

impl http::Server for Component {
    fn handle(
        request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        let (parts, _) = request.into_parts();

        let Some(path_with_query) = parts.uri.path_and_query() else {
            return http::Response::builder()
                .status(400)
                .body("Bad request, did not contain path and query".into())
                .map_err(|e| ErrorCode::InternalError(Some(e.to_string())));
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
        .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;

        // Open the KV bucket
        let bucket = store::open("default")
            .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;

        let trimmed_path = object_name.trim_start_matches('/');

        // Handle GET / - return info message
        if trimmed_path.is_empty() && method == http::Method::GET {
            return json_response(
                &InfoMessage {
                    message: "Counter service. Use POST /:name to increment, GET /:name to read.".into(),
                },
                200,
            );
        }

        // Reject empty paths
        if trimmed_path.is_empty() {
            return json_response(&ErrorMessage { error: "Not found".into() }, 404);
        }

        // Handle GET /:name - read counter by incrementing by 0
        if method == http::Method::GET {
            return match atomics::increment(&bucket, trimmed_path, 0) {
                Ok(value) => json_response(
                    &CounterData {
                        name: trimmed_path.into(),
                        value,
                    },
                    200,
                ),
                Err(_) => json_response(&ErrorMessage { error: "Counter not found".into() }, 404),
            };
        }

        // Handle POST /:name - increment counter
        if method != http::Method::POST {
            return json_response(&ErrorMessage { error: "Method not allowed".into() }, 405);
        }

        let count = atomics::increment(&bucket, trimmed_path, 1)
            .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;

        json_response(
            &CounterData {
                name: trimmed_path.into(),
                value: count,
            },
            200,
        )
    }
}
