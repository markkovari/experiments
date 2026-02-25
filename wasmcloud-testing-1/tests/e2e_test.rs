// End-to-end tests for HTTP KV Counter component
// These tests require a running wasmCloud host and NATS server
//
// To run these tests:
// 1. Start NATS server: `nats-server -js`
// 2. Start wasmCloud host: `wash up`
// 3. Deploy the application: `wash app deploy wadm.yaml`
// 4. Run tests: `cargo test --test e2e_test -- --ignored`

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CounterData {
    name: String,
    value: i64,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    error: String,
}

// Base URL for the deployed component
const BASE_URL: &str = "http://localhost:8080";

/// Test GET / endpoint - returns info message
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn test_get_root_info() {
    let client = reqwest::Client::new();

    let response = client
        .get(BASE_URL)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");

    // Should return an info message
    assert!(json["message"].as_str().is_some());
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Counter service"));
}

/// Test POST /:name endpoint - creates and increments counter
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn test_post_increment_counter() {
    let client = reqwest::Client::new();
    let counter_name = format!("test_counter_{}", chrono::Utc::now().timestamp());

    // First increment - should create with value 1
    let response = client
        .post(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");

    assert_eq!(counter.name, counter_name);
    assert_eq!(counter.value, 1);

    // Second increment - should return value 2
    let response = client
        .post(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");

    assert_eq!(counter.name, counter_name);
    assert_eq!(counter.value, 2);
}

/// Test GET /:name endpoint - retrieves specific counter
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn test_get_specific_counter() {
    let client = reqwest::Client::new();
    let counter_name = format!("get_test_{}", chrono::Utc::now().timestamp());

    // Create counter first
    client
        .post(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to create counter");

    // Retrieve the counter
    let response = client
        .get(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");

    assert_eq!(counter.name, counter_name);
    assert_eq!(counter.value, 1);
}

/// Test GET /:name endpoint - returns 0 for non-existent counter
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn test_get_nonexistent_counter() {
    let client = reqwest::Client::new();
    let counter_name = format!("nonexistent_{}", chrono::Utc::now().timestamp());

    let response = client
        .get(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");

    assert_eq!(counter.name, counter_name);
    assert_eq!(counter.value, 0);
}

/// Test TTL functionality - counter should expire after 3 seconds
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn test_counter_ttl_expiration() {
    let client = reqwest::Client::new();
    let counter_name = format!("ttl_test_{}", chrono::Utc::now().timestamp());

    println!("Creating counter: {}", counter_name);

    // Create counter
    let response = client
        .post(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to create counter");

    assert_eq!(response.status(), 200);

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");

    assert_eq!(counter.value, 1);

    println!("Counter created with value: {}", counter.value);

    // Verify counter exists immediately
    let response = client
        .get(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to get counter");

    assert_eq!(response.status(), 200);
    println!("Counter still exists immediately after creation");

    // Wait for TTL to expire (3 seconds + 1 second buffer)
    println!("Waiting 4 seconds for TTL expiration...");
    tokio::time::sleep(Duration::from_secs(4)).await;

    // Verify counter has been reset to 0
    let response = client
        .get(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to get counter");

    println!("Response status after TTL: {}", response.status());

    assert_eq!(response.status(), 200);

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");

    // Should return 0 after TTL expiration (counter resets)
    assert_eq!(counter.value, 0);
    println!("Counter successfully expired after TTL (value reset to 0)");
}

/// Test multiple independent counters
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn test_multiple_counters() {
    let client = reqwest::Client::new();
    let timestamp = chrono::Utc::now().timestamp();

    let counter_names = vec![
        format!("multi_1_{}", timestamp),
        format!("multi_2_{}", timestamp),
        format!("multi_3_{}", timestamp),
    ];

    // Create multiple counters
    for name in &counter_names {
        let response = client
            .post(&format!("{}/{}", BASE_URL, name))
            .send()
            .await
            .expect("Failed to create counter");

        assert_eq!(response.status(), 200);

        let counter: CounterData = response.json().await.expect("Failed to parse JSON");

        assert_eq!(counter.name, *name);
        assert_eq!(counter.value, 1);
    }

    // Verify each counter independently
    for name in &counter_names {
        let response = client
            .get(&format!("{}/{}", BASE_URL, name))
            .send()
            .await
            .expect("Failed to get counter");

        assert_eq!(response.status(), 200);

        let counter: CounterData = response.json().await.expect("Failed to parse JSON");

        assert_eq!(counter.name, *name);
        assert_eq!(counter.value, 1);
    }
}

/// Test concurrent increments
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn test_concurrent_increments() {
    let client = reqwest::Client::new();
    let counter_name = format!("concurrent_{}", chrono::Utc::now().timestamp());

    // Perform 10 concurrent increments with small delays
    let mut handles = vec![];

    for _ in 0..10 {
        let client_clone = client.clone();
        let name_clone = counter_name.clone();

        let handle = tokio::spawn(async move {
            client_clone
                .post(&format!("{}/{}", BASE_URL, name_clone))
                .send()
                .await
                .expect("Failed to increment counter")
        });

        handles.push(handle);
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    // Verify final counter value
    let response = client
        .get(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to get counter");

    assert_eq!(response.status(), 200);

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");

    // Should have incremented to 10 (with proper atomics)
    assert_eq!(
        counter.value, 10,
        "Concurrent increments failed, expected 10 got {}",
        counter.value
    );
}

/// Test invalid HTTP methods
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn test_invalid_http_methods() {
    let client = reqwest::Client::new();
    let counter_name = "test_invalid_method";

    // Try DELETE (should return 404 or 405)
    let response = client
        .delete(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status() == 404 || response.status() == 405);

    // Try PUT (should return 404 or 405)
    let response = client
        .put(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status() == 404 || response.status() == 405);
}

/// Test counter with special characters in name
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn test_counter_special_characters() {
    let client = reqwest::Client::new();
    let counter_name = format!("counter-with-dashes_{}", chrono::Utc::now().timestamp());

    // Create counter with dashes
    let response = client
        .post(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to create counter");

    assert_eq!(response.status(), 200);

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");

    assert_eq!(counter.name, counter_name);
    assert_eq!(counter.value, 1);
}
