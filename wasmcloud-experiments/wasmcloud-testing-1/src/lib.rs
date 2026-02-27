use serde::Serialize;
use wasmcloud_component::http;
use wasmcloud_component::http::ErrorCode;
use wasmcloud_component::wasi::keyvalue::*;

const BUCKET_NAME: &str = "default";

#[derive(Serialize)]
struct CounterData {
    name: String,
    value: u64,
}

#[derive(Serialize)]
struct InfoMessage {
    message: String,
}

impl Default for InfoMessage {
    fn default() -> Self {
        Self {
            message: "Counter service. Use POST /:name to increment, GET /:name to read.".into(),
        }
    }
}

#[derive(Serialize)]
struct ErrorMessage {
    error: String,
}

impl Default for ErrorMessage {
    fn default() -> Self {
        Self {
            error: "Unknown error".into(),
        }
    }
}

struct Component;

http::export!(Component);

/// Helper function to create a JSON response with proper error handling
fn json_response<T: Serialize>(
    data: &T,
    status: http::StatusCode,
) -> http::Result<http::Response<String>> {
    let mut json =
        serde_json::to_string(data).map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
    json.push('\n');

    http::Response::builder()
        .status(status)
        .body(json)
        .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))
}

/// Response builders
fn info_response() -> http::Result<http::Response<String>> {
    json_response(&InfoMessage::default(), http::StatusCode::OK)
}

fn counter_response(name: &str, value: u64) -> http::Result<http::Response<String>> {
    json_response(
        &CounterData {
            name: name.into(),
            value,
        },
        http::StatusCode::OK,
    )
}

fn not_found() -> http::Result<http::Response<String>> {
    json_response(
        &ErrorMessage {
            error: "Not found".into(),
        },
        http::StatusCode::NOT_FOUND,
    )
}

fn method_not_allowed() -> http::Result<http::Response<String>> {
    json_response(
        &ErrorMessage {
            error: "Method not allowed".into(),
        },
        http::StatusCode::METHOD_NOT_ALLOWED,
    )
}

fn bad_request() -> http::Result<http::Response<String>> {
    http::Response::builder()
        .status(http::StatusCode::BAD_REQUEST)
        .body("Bad request, did not contain path and query".into())
        .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))
}

/// Counter operations
fn get_counter(bucket: &store::Bucket, name: &str) -> http::Result<u64> {
    atomics::increment(bucket, name, 0).map_err(|e| ErrorCode::InternalError(Some(e.to_string())))
}

fn increment_counter(bucket: &store::Bucket, name: &str) -> http::Result<u64> {
    atomics::increment(bucket, name, 1).map_err(|e| ErrorCode::InternalError(Some(e.to_string())))
}

impl http::Server for Component {
    fn handle(
        request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        let (parts, _) = request.into_parts();
        let uri = parts.uri;
        let method = parts.method;

        let Some(path_with_query) = uri.path_and_query() else {
            return bad_request();
        };

        let path = path_with_query.path();
        let trimmed_path = path.trim_start_matches('/');

        match (&method, trimmed_path) {
            (&http::Method::GET, "") => info_response(),
            (_, "") => not_found(),
            (&http::Method::PUT | &http::Method::DELETE | &http::Method::PATCH, _) => {
                method_not_allowed()
            }
            (&http::Method::GET, counter_name) => {
                let bucket = store::open(BUCKET_NAME)
                    .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
                counter_response(counter_name, get_counter(&bucket, counter_name)?)
            }
            (&http::Method::POST, counter_name) => {
                let bucket = store::open(BUCKET_NAME)
                    .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
                counter_response(counter_name, increment_counter(&bucket, counter_name)?)
            }
            _ => method_not_allowed(),
        }
    }
}
