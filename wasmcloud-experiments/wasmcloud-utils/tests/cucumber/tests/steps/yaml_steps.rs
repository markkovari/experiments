use crate::WorkflowWorld;
use cucumber::when;
use cucumber::gherkin::Step;

/// POST with an explicit content-type (used for YAML tests).
#[when(expr = "I POST to {string} with content-type {string} and body:")]
async fn post_with_content_type(
    world: &mut WorkflowWorld,
    path: String,
    content_type: String,
    step: &Step,
) {
    let body = step.docstring().map_or("", |v| v.as_str()).to_string();
    let url = format!("{}{}", world.base_url, path);
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", content_type)
        .body(body)
        .send()
        .await
        .expect("HTTP POST failed");
    world.last_status = resp.status().as_u16();
    world.last_body = resp.text().await.unwrap_or_default();
}
