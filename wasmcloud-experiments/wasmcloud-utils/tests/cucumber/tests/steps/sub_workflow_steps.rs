use crate::WorkflowWorld;
use cucumber::{given, then};

/// Simulate child run success by marking the sub-workflow step as done.
#[given(expr = "the child run for step {string} on run {string} succeeds")]
async fn child_run_succeeds(
    world: &mut WorkflowWorld,
    step_name: String,
    _placeholder: String,
) {
    let run_id = world.run_id.clone().expect("no run_id");
    let url = format!(
        "{}/runs/{}/steps/{}/done",
        world.base_url, run_id, step_name
    );
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", "application/json")
        .body(r#"{"output":""}"#)
        .send()
        .await
        .expect("failed to mark sub-workflow step done");
    assert_eq!(resp.status().as_u16(), 200);
}

/// Simulate child run failure.
#[given(expr = "the child run for step {string} on run {string} fails")]
async fn child_run_fails(
    world: &mut WorkflowWorld,
    step_name: String,
    _placeholder: String,
) {
    let run_id = world.run_id.clone().expect("no run_id");
    let url = format!(
        "{}/runs/{}/steps/{}/failed",
        world.base_url, run_id, step_name
    );
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", "application/json")
        .body(r#"{"error":"child failed"}"#)
        .send()
        .await
        .expect("failed to mark sub-workflow step failed");
    assert_eq!(resp.status().as_u16(), 200);
}

/// Check a specific step's state via GET /runs/{run_id}/steps/{step_name}.
#[then(expr = "the step {string} state is {string}")]
async fn check_step_state(world: &mut WorkflowWorld, step_name: String, expected_state: String) {
    let run_id = world.run_id.clone().expect("no run_id");
    let url = format!(
        "{}/runs/{}/steps/{}",
        world.base_url, run_id, step_name
    );
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .expect("failed to get step");
    let body = resp.text().await.unwrap_or_default();
    assert!(
        body.contains(&expected_state),
        "Expected step '{}' to have state '{}', got: {}",
        step_name,
        expected_state,
        body
    );
}
