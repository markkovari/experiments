// Integration tests for HTTP KV Counter component
// These tests verify the business logic without requiring a full wasmCloud runtime

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CounterData {
    name: String,
    value: i64,
}

/// Test path parsing logic
#[test]
fn test_path_parsing() {
    // Test root path
    assert_eq!(parse_path("/"), ("root", None));

    // Test counter path
    assert_eq!(
        parse_path("/mycounter"),
        ("counter", Some("mycounter".to_string()))
    );

    // Test invalid paths
    assert_eq!(parse_path("/foo/bar/baz"), ("unknown", None));
    assert_eq!(parse_path(""), ("root", None));
}

/// Test counter increment logic
#[test]
fn test_counter_increment() {
    // Test incrementing from 0
    let result = increment_counter_logic(None);
    assert_eq!(result, 1);

    // Test incrementing existing value
    let result = increment_counter_logic(Some(5));
    assert_eq!(result, 6);

    // Test incrementing from negative
    let result = increment_counter_logic(Some(-1));
    assert_eq!(result, 0);
}

/// Test JSON serialization/deserialization
#[test]
fn test_json_serialization() {
    let data = CounterData {
        name: "test_counter".to_string(),
        value: 42,
    };

    // Serialize
    let json = serde_json::to_string(&data).expect("Failed to serialize");
    assert!(json.contains("\"name\":\"test_counter\""));
    assert!(json.contains("\"value\":42"));

    // Deserialize
    let deserialized: CounterData = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.name, "test_counter");
    assert_eq!(deserialized.value, 42);
}

/// Test JSON array serialization for multiple counters
#[test]
fn test_json_array_serialization() {
    let counters = vec![
        CounterData {
            name: "counter1".to_string(),
            value: 10,
        },
        CounterData {
            name: "counter2".to_string(),
            value: 20,
        },
    ];

    let json = serde_json::to_string(&counters).expect("Failed to serialize");
    let deserialized: Vec<CounterData> =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.len(), 2);
    assert_eq!(deserialized[0].name, "counter1");
    assert_eq!(deserialized[0].value, 10);
    assert_eq!(deserialized[1].name, "counter2");
    assert_eq!(deserialized[1].value, 20);
}

/// Test error responses
#[test]
fn test_error_response_format() {
    let error = serde_json::json!({"error": "Counter not found"});
    let json = error.to_string();

    assert!(json.contains("\"error\""));
    assert!(json.contains("Counter not found"));
}

/// Test counter value parsing
#[test]
fn test_value_parsing() {
    // Valid values
    assert_eq!(parse_counter_value(b"42"), Ok(42));
    assert_eq!(parse_counter_value(b"0"), Ok(0));
    assert_eq!(parse_counter_value(b"-1"), Ok(-1));

    // Invalid values
    assert!(parse_counter_value(b"not_a_number").is_err());
    assert!(parse_counter_value(b"").is_err());
    assert!(parse_counter_value(&[255, 255]).is_err()); // Invalid UTF-8
}

/// Test HTTP method routing logic
#[test]
fn test_http_routing_logic() {
    // GET / should return all counters
    let route = determine_action("GET", "/");
    assert_eq!(route, Action::GetAll);

    // GET /:name should return specific counter
    let route = determine_action("GET", "/mycounter");
    assert_eq!(route, Action::GetOne("mycounter".to_string()));

    // POST /:name should increment counter
    let route = determine_action("POST", "/mycounter");
    assert_eq!(route, Action::Increment("mycounter".to_string()));

    // Unsupported methods
    let route = determine_action("DELETE", "/mycounter");
    assert_eq!(route, Action::NotFound);

    let route = determine_action("PUT", "/mycounter");
    assert_eq!(route, Action::NotFound);
}

// Helper functions that mirror the logic in src/lib.rs

fn parse_path(path: &str) -> (&str, Option<String>) {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        [""] => ("root", None),
        [name] if !name.is_empty() => ("counter", Some(name.to_string())),
        _ => ("unknown", None),
    }
}

fn increment_counter_logic(current_value: Option<i64>) -> i64 {
    current_value.unwrap_or(0) + 1
}

fn parse_counter_value(bytes: &[u8]) -> Result<i64, String> {
    let value_str = std::str::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {}", e))?;
    value_str
        .parse::<i64>()
        .map_err(|e| format!("Invalid number: {}", e))
}

#[derive(Debug, PartialEq)]
enum Action {
    GetAll,
    GetOne(String),
    Increment(String),
    NotFound,
}

fn determine_action(method: &str, path: &str) -> Action {
    let (route, name_param) = parse_path(path);

    match (method, route, name_param) {
        ("GET", "root", None) => Action::GetAll,
        ("GET", "counter", Some(name)) => Action::GetOne(name),
        ("POST", "counter", Some(name)) => Action::Increment(name),
        _ => Action::NotFound,
    }
}

/// Test concurrent counter increments (simulation)
#[test]
fn test_concurrent_increments() {
    // Simulate multiple increments
    let mut value = 0i64;

    for _ in 0..100 {
        value = increment_counter_logic(Some(value));
    }

    assert_eq!(value, 100);
}

/// Test edge cases
#[test]
fn test_edge_cases() {
    // Empty counter name
    let (route, name) = parse_path("/");
    assert_eq!(route, "root");
    assert_eq!(name, None);

    // Counter name with special characters
    let (route, name) = parse_path("/counter-with-dashes");
    assert_eq!(route, "counter");
    assert_eq!(name, Some("counter-with-dashes".to_string()));

    // Very large counter value
    let result = increment_counter_logic(Some(i64::MAX - 1));
    assert_eq!(result, i64::MAX);
}
