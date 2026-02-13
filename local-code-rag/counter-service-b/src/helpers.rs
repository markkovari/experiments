use wasmcloud_component::http;
use wasmcloud_component::http::ErrorCode;

pub fn extract_key(request: &http::IncomingRequest) -> Result<String, ErrorCode> {
    let (parts, _) = request.into_parts();

    let path_with_query = parts.uri.path_and_query().ok_or_else(|| {
        ErrorCode::InternalError(Some("request has no path".to_string()))
    })?;

    let key = path_with_query
        .path()
        .trim_start_matches('/')
        .to_string();

    if key.is_empty() {
        return Err(ErrorCode::InternalError(Some(
            "path is empty, provide a counter name in the URL".to_string(),
        )));
    }

    Ok(key)
}

pub fn format_response(key: &str, count: u64) -> String {
    format!("{{\"key\": \"{key}\", \"count\": {count}}}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_response() {
        let result = format_response("foo", 42);
        assert_eq!(result, "{\"key\": \"foo\", \"count\": 42}\n");
    }
}
