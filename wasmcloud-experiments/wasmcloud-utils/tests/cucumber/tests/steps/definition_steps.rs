use crate::{WorkflowWorld, api_available};
use cucumber::given;
use cucumber::gherkin::Step;

/// Register a minimal workflow with the given name.
#[given(expr = "I have registered a workflow named {string}")]
async fn register_workflow(world: &mut WorkflowWorld, name: String) {
    if !api_available().await {
        return;
    }
    let body = format!(
        r#"{{"name":"{}","steps":[{{"name":"step","depends_on":[]}}]}}"#,
        name
    );
    let url = format!("{}/workflows", world.base_url);
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .expect("failed to register workflow");
    let status = resp.status().as_u16();
    assert!(
        status == 201 || status == 200,
        "Failed to register workflow '{}': status {}",
        name,
        status
    );
}

/// Register a workflow with a specific steps JSON array.
#[given(expr = "I have registered a workflow named {string} with steps:")]
async fn register_workflow_with_steps(
    world: &mut WorkflowWorld,
    name: String,
    step: &Step,
) {
    if !api_available().await {
        return;
    }
    let steps_json = step.docstring().map_or("[]", |v| v.as_str());
    let body = format!(r#"{{"name":"{}","steps":{}}}"#, name, steps_json);
    let url = format!("{}/workflows", world.base_url);
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .expect("failed to register workflow");
    let status = resp.status().as_u16();
    assert!(
        status == 201 || status == 200,
        "Failed to register workflow '{}': status {}",
        name,
        status
    );
}
