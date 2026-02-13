mod helpers;

use wasmcloud_component::http;
use wasmcloud_component::http::ErrorCode;
use wasmcloud_component::wasi::keyvalue::*;

use helpers::{extract_key, format_response};

struct KvCounter;

http::export!(KvCounter);

impl http::Server for KvCounter {
    fn handle(
        request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        let key = extract_key(&request)?;
        let count = increment_counter(&key)?;
        Ok(http::Response::new(format_response(&key, count)))
    }
}

fn increment_counter(key: &str) -> Result<u64, ErrorCode> {
    let bucket = store::open("default").map_err(|e| {
        ErrorCode::InternalError(Some(format!("failed to open kv bucket: {e:?}")))
    })?;

    atomics::increment(&bucket, key, 1).map_err(|e| {
        ErrorCode::InternalError(Some(format!("failed to increment key '{key}': {e:?}")))
    })
}
