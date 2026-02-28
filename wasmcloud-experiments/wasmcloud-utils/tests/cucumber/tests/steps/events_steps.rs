use crate::{WorkflowWorld, api_available};
use cucumber::given;

/// Subscribe a function to an event (setup step).
#[given(expr = "I have subscribed {string} to event {string}")]
async fn subscribe_to_event(world: &mut WorkflowWorld, fn_name: String, event: String) {
    if !api_available().await {
        return;
    }
    let body = format!(r#"{{"fn_name":"{}"}}"#, fn_name);
    let url = format!("{}/events/{}/subscribe", world.base_url, event);
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .expect("failed to subscribe to event");
    let status = resp.status().as_u16();
    assert_eq!(status, 200, "Expected 200 from subscribe endpoint");
}
