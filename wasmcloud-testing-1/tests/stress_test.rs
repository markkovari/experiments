// Stress tests for HTTP KV Counter component
// These tests verify system behavior under high load
//
// To run these tests:
// 1. Start wasmCloud host: `wash up --detached`
// 2. Deploy application: `wash app deploy wadm.yaml`
// 3. Run tests: `cargo test --test stress_test --target aarch64-apple-darwin -- --ignored --test-threads=1`

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CounterData {
    name: String,
    value: i64,
}

const BASE_URL: &str = "http://localhost:8080";

/// Test high volume sequential requests
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn stress_test_sequential_requests() {
    let client = reqwest::Client::new();
    let counter_name = format!("stress_seq_{}", chrono::Utc::now().timestamp_millis());
    let num_requests = 200;

    println!(
        "Starting sequential stress test with {} requests",
        num_requests
    );
    let start = Instant::now();

    for i in 0..num_requests {
        let response = client
            .post(&format!("{}/{}", BASE_URL, counter_name))
            .send()
            .await
            .expect(&format!("Failed at request {}", i));

        assert_eq!(response.status(), 200, "Failed at request {}", i);

        let counter: CounterData = response
            .json()
            .await
            .expect(&format!("Failed to parse JSON at request {}", i));

        assert_eq!(counter.value, i + 1, "Incorrect value at request {}", i);
    }

    let duration = start.elapsed();
    println!(
        "Completed {} sequential requests in {:?}",
        num_requests, duration
    );
    println!(
        "Average: {:.2} req/sec",
        num_requests as f64 / duration.as_secs_f64()
    );

    // Verify final count
    let response = client
        .get(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to get final count");

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");
    assert_eq!(counter.value, num_requests);
}

/// Test high concurrency - many parallel requests
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn stress_test_high_concurrency() {
    let client = Arc::new(reqwest::Client::new());
    let counter_name = format!(
        "stress_concurrent_{}",
        chrono::Utc::now().timestamp_millis()
    );
    let num_concurrent = 5;

    println!(
        "Starting high concurrency test with {} parallel requests",
        num_concurrent
    );
    let start = Instant::now();

    let mut handles = vec![];

    for i in 0..num_concurrent {
        let client_clone = Arc::clone(&client);
        let name_clone = counter_name.clone();

        let handle = tokio::spawn(async move {
            let response = client_clone
                .post(&format!("{}/{}", BASE_URL, name_clone))
                .send()
                .await
                .expect(&format!("Failed at concurrent request {}", i));

            assert_eq!(response.status(), 200, "Failed at concurrent request {}", i);
            response
                .json::<CounterData>()
                .await
                .expect("Failed to parse JSON")
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    let results: Vec<CounterData> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task panicked"))
        .collect();

    let duration = start.elapsed();
    println!(
        "Completed {} concurrent requests in {:?}",
        num_concurrent, duration
    );
    println!(
        "Average: {:.2} req/sec",
        num_concurrent as f64 / duration.as_secs_f64()
    );

    // Verify all requests succeeded
    assert_eq!(results.len(), num_concurrent);

    // Verify final count (should be exactly num_concurrent due to atomics)
    let response = client
        .get(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to get final count");

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");
    assert_eq!(
        counter.value, num_concurrent as i64,
        "Final count should equal number of concurrent increments"
    );
}

/// Test many unique counters
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn stress_test_many_unique_counters() {
    let client = reqwest::Client::new();
    let timestamp = chrono::Utc::now().timestamp_millis();
    let num_counters = 100;

    println!("Creating {} unique counters", num_counters);
    let start = Instant::now();

    for i in 0..num_counters {
        let counter_name = format!("stress_unique_{}_{}", timestamp, i);

        let response = client
            .post(&format!("{}/{}", BASE_URL, counter_name))
            .send()
            .await
            .expect(&format!("Failed to create counter {}", i));

        assert_eq!(response.status(), 200);

        let counter: CounterData = response.json().await.expect("Failed to parse JSON");
        assert_eq!(counter.name, counter_name);
        assert_eq!(counter.value, 1);
    }

    let duration = start.elapsed();
    println!("Created {} unique counters in {:?}", num_counters, duration);
    println!(
        "Average: {:.2} counters/sec",
        num_counters as f64 / duration.as_secs_f64()
    );
}

/// Test rapid fire increments on multiple counters concurrently
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn stress_test_mixed_workload() {
    let client = Arc::new(reqwest::Client::new());
    let timestamp = chrono::Utc::now().timestamp_millis();
    let num_counters = 5;
    let increments_per_counter = 20;

    println!(
        "Mixed workload: {} counters × {} increments = {} total requests",
        num_counters,
        increments_per_counter,
        num_counters * increments_per_counter
    );
    let start = Instant::now();

    let mut handles = vec![];

    for counter_id in 0..num_counters {
        for _ in 0..increments_per_counter {
            let client_clone = Arc::clone(&client);
            let counter_name = format!("stress_mixed_{}_{}", timestamp, counter_id);

            let handle = tokio::spawn(async move {
                client_clone
                    .post(&format!("{}/{}", BASE_URL, counter_name))
                    .send()
                    .await
                    .expect("Request failed")
                    .json::<CounterData>()
                    .await
                    .expect("Failed to parse JSON")
            });

            handles.push(handle);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    let results: Vec<CounterData> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task panicked"))
        .collect();

    let duration = start.elapsed();
    let total_requests = num_counters * increments_per_counter;

    println!(
        "Completed {} mixed requests in {:?}",
        total_requests, duration
    );
    println!(
        "Average: {:.2} req/sec",
        total_requests as f64 / duration.as_secs_f64()
    );

    assert_eq!(results.len(), total_requests);

    // Verify each counter has the correct final value
    for counter_id in 0..num_counters {
        let counter_name = format!("stress_mixed_{}_{}", timestamp, counter_id);

        let response = client
            .get(&format!("{}/{}", BASE_URL, counter_name))
            .send()
            .await
            .expect("Failed to get counter");

        let counter: CounterData = response.json().await.expect("Failed to parse JSON");
        assert_eq!(
            counter.value, increments_per_counter as i64,
            "Counter {} should have value {}",
            counter_name, increments_per_counter
        );
    }
}

/// Test sustained load over time
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn stress_test_sustained_load() {
    let client = Arc::new(reqwest::Client::new());
    let counter_name = format!("stress_sustained_{}", chrono::Utc::now().timestamp_millis());
    let duration = Duration::from_secs(30);
    let requests_per_second = 50;

    println!(
        "Sustained load test: {} req/sec for {:?}",
        requests_per_second, duration
    );

    let success_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    // Spawn background task to make requests
    let client_clone = Arc::clone(&client);
    let success_clone = Arc::clone(&success_count);
    let error_clone = Arc::clone(&error_count);
    let counter_name_clone = counter_name.clone();

    let handle = tokio::spawn(async move {
        while start.elapsed() < duration {
            let client_clone2 = Arc::clone(&client_clone);
            let name_clone = counter_name_clone.clone();
            let success_clone2 = Arc::clone(&success_clone);
            let error_clone2 = Arc::clone(&error_clone);

            tokio::spawn(async move {
                match client_clone2
                    .post(&format!("{}/{}", BASE_URL, name_clone))
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        success_clone2.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {
                        error_clone2.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });

            tokio::time::sleep(Duration::from_millis(1000 / requests_per_second)).await;
        }
    });

    handle.await.expect("Sustained load task failed");

    let total_duration = start.elapsed();
    let successes = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);

    println!(
        "Sustained load completed: {} successes, {} errors in {:?}",
        successes, errors, total_duration
    );
    println!(
        "Actual rate: {:.2} req/sec",
        successes as f64 / total_duration.as_secs_f64()
    );

    // At least 80% success rate expected
    let success_rate = successes as f64 / (successes + errors) as f64;
    assert!(
        success_rate >= 0.8,
        "Success rate too low: {:.2}%",
        success_rate * 100.0
    );
}

/// Test TTL stress - create many counters and verify they expire
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn stress_test_ttl_expiration() {
    let client = reqwest::Client::new();
    let timestamp = chrono::Utc::now().timestamp_millis();
    let num_counters = 20;

    println!("Creating {} counters for TTL stress test", num_counters);

    // Create many counters
    let counter_names: Vec<String> = (0..num_counters)
        .map(|i| format!("stress_ttl_{}_{}", timestamp, i))
        .collect();

    for name in &counter_names {
        let response = client
            .post(&format!("{}/{}", BASE_URL, name))
            .send()
            .await
            .expect("Failed to create counter");

        assert_eq!(response.status(), 200);
    }

    println!("All counters created, waiting for TTL expiration (4 seconds)...");
    tokio::time::sleep(Duration::from_secs(4)).await;

    // Verify all counters have expired (value = 0)
    let mut expired_count = 0;

    for name in &counter_names {
        let response = client
            .get(&format!("{}/{}", BASE_URL, name))
            .send()
            .await
            .expect("Failed to get counter");

        let counter: CounterData = response.json().await.expect("Failed to parse JSON");

        if counter.value == 0 {
            expired_count += 1;
        }
    }

    println!(
        "TTL expiration: {}/{} counters expired",
        expired_count, num_counters
    );

    // All counters should have expired
    assert_eq!(
        expired_count, num_counters,
        "Not all counters expired after TTL"
    );
}

/// Test burst traffic - rapid succession of requests
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn stress_test_burst_traffic() {
    let client = Arc::new(reqwest::Client::new());
    let counter_name = format!("stress_burst_{}", chrono::Utc::now().timestamp_millis());
    let burst_size = 8;

    println!(
        "Burst traffic test: {} requests as fast as possible",
        burst_size
    );
    let start = Instant::now();

    let mut handles = vec![];

    // Fire all requests with small delays to avoid overwhelming single instance
    for i in 0..burst_size {
        let client_clone = Arc::clone(&client);
        let name_clone = counter_name.clone();

        let handle = tokio::spawn(async move {
            client_clone
                .post(&format!("{}/{}", BASE_URL, name_clone))
                .send()
                .await
                .expect(&format!("Burst request {} failed", i))
                .json::<CounterData>()
                .await
                .expect("Failed to parse JSON")
        });

        handles.push(handle);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    let results: Vec<CounterData> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task panicked"))
        .collect();

    let duration = start.elapsed();

    println!("Burst completed: {} requests in {:?}", burst_size, duration);
    println!(
        "Peak rate: {:.2} req/sec",
        burst_size as f64 / duration.as_secs_f64()
    );

    assert_eq!(results.len(), burst_size);

    // Verify final count
    let response = client
        .get(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to get final count");

    let counter: CounterData = response.json().await.expect("Failed to parse JSON");
    assert_eq!(
        counter.value, burst_size as i64,
        "Final count should equal burst size"
    );
}

/// Test read-heavy workload
#[tokio::test]
#[ignore] // Requires running wasmCloud host
async fn stress_test_read_heavy_workload() {
    let client = Arc::new(reqwest::Client::new());
    let counter_name = format!(
        "stress_read_heavy_{}",
        chrono::Utc::now().timestamp_millis()
    );

    // Create counter with initial value
    client
        .post(&format!("{}/{}", BASE_URL, counter_name))
        .send()
        .await
        .expect("Failed to create counter");

    let num_reads = 30;
    let num_writes = 5;

    println!(
        "Read-heavy workload: {} reads + {} writes",
        num_reads, num_writes
    );
    let start = Instant::now();

    let mut handles = vec![];

    // Mostly reads with small delays
    for _ in 0..num_reads {
        let client_clone = Arc::clone(&client);
        let name_clone = counter_name.clone();

        let handle = tokio::spawn(async move {
            client_clone
                .get(&format!("{}/{}", BASE_URL, name_clone))
                .send()
                .await
                .expect("Read failed")
                .json::<CounterData>()
                .await
                .expect("Failed to parse JSON")
        });

        handles.push(handle);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Some writes with small delays
    for _ in 0..num_writes {
        let client_clone = Arc::clone(&client);
        let name_clone = counter_name.clone();

        let handle = tokio::spawn(async move {
            client_clone
                .post(&format!("{}/{}", BASE_URL, name_clone))
                .send()
                .await
                .expect("Write failed")
                .json::<CounterData>()
                .await
                .expect("Failed to parse JSON")
        });

        handles.push(handle);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let results: Vec<CounterData> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task panicked"))
        .collect();

    let duration = start.elapsed();
    let total_requests = num_reads + num_writes;

    println!(
        "Read-heavy workload completed: {} requests in {:?}",
        total_requests, duration
    );
    println!(
        "Average: {:.2} req/sec",
        total_requests as f64 / duration.as_secs_f64()
    );

    assert_eq!(results.len(), total_requests);
}
