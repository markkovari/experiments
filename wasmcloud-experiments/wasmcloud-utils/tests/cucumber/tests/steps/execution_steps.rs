use crate::{WorkflowWorld, api_available};
use cucumber::given;

/// Start a run of the given workflow, storing the run_id.
#[given(expr = "I have started a run of {string}")]
async fn start_run(world: &mut WorkflowWorld, wf_name: String) {
    if !api_available().await {
        return;
    }
    let body = format!(r#"{{"wf_name":"{}"}}"#, wf_name);
    let url = format!("{}/runs", world.base_url);
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .expect("failed to start run");
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    assert_eq!(status, 201, "Failed to start run: {}", text);
    let json: serde_json::Value = serde_json::from_str(&text).expect("invalid JSON from /runs");
    let run_id = json["run_id"]
        .as_str()
        .expect("no run_id in /runs response")
        .to_string();
    world.run_id = Some(run_id);
    world.last_status = status;
    world.last_body = text;
}

/// Mark a step as failed so retry can be tested.
#[given(expr = "step {string} has failed on run {string}")]
async fn step_has_failed(world: &mut WorkflowWorld, step_name: String, _placeholder: String) {
    if !api_available().await {
        return;
    }
    let run_id = world.run_id.clone().expect("no run_id set");
    let url = format!(
        "{}/runs/{}/steps/{}/failed",
        world.base_url, run_id, step_name
    );
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", "application/json")
        .body(r#"{"error":"test failure"}"#)
        .send()
        .await
        .expect("failed to mark step as failed");
    let s = resp.status().as_u16();
    assert!(s == 200 || s == 204, "Expected 200/204 from step failed, got {}", s);
}

/// Mark a step as done with the given base64 output.
/// The output_b64 string is decoded from base64 to get the raw bytes,
/// then sent as a JSON array of byte numbers (matching the legacy Vec<u8> format).
#[given(expr = "I mark step {string} as done with output {string} on run {string}")]
async fn mark_step_done(
    world: &mut WorkflowWorld,
    step_name: String,
    output_b64: String,
    _placeholder: String,
) {
    if !api_available().await {
        return;
    }
    use base64::Engine;
    let run_id = world.run_id.clone().expect("no run_id set");
    let url = format!(
        "{}/runs/{}/steps/{}/done",
        world.base_url, run_id, step_name
    );
    // Decode base64 to bytes, then encode as JSON number array for Vec<u8> compat
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&output_b64)
        .unwrap_or_else(|_| output_b64.as_bytes().to_vec());
    let nums: Vec<u8> = bytes;
    let arr = serde_json::to_string(&nums).unwrap();
    let body = format!(r#"{{"output":{}}}"#, arr);
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .expect("failed to mark step as done");
    let s = resp.status().as_u16();
    assert!(s == 200 || s == 204, "Expected 200/204 from step done, got {}", s);
}
