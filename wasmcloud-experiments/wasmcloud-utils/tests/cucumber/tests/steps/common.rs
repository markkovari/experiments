use crate::{WorkflowWorld, api_available};
use cucumber::{given, then, when};
use cucumber::gherkin::Step;

#[given(expr = "the workflow API is running at {string}")]
async fn set_base_url(world: &mut WorkflowWorld, url: String) {
    world.base_url = url;
}

#[when(expr = "I GET {string}")]
async fn get_endpoint(world: &mut WorkflowWorld, path: String) {
    if !api_available().await {
        return;
    }
    let path = expand_run_id(&path, &world.run_id);
    let url = format!("{}{}", world.base_url, path);
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .expect("HTTP GET failed");
    world.last_status = resp.status().as_u16();
    world.last_body = resp.text().await.unwrap_or_default();
}

#[when(expr = "I POST to {string} with body:")]
async fn post_with_body(world: &mut WorkflowWorld, path: String, step: &Step) {
    if !api_available().await {
        return;
    }
    let path = expand_run_id(&path, &world.run_id);
    let body = step.docstring().map_or("{}", |v| v.as_str()).to_string();
    let url = format!("{}{}", world.base_url, path);
    let resp = reqwest::Client::new()
        .post(&url)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .expect("HTTP POST failed");
    world.last_status = resp.status().as_u16();
    world.last_body = resp.text().await.unwrap_or_default();

    // Auto-save run_id if present in response
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&world.last_body) {
        if let Some(id) = json["run_id"].as_str() {
            world.run_id = Some(id.to_string());
        }
    }
}

#[when(expr = "I DELETE {string}")]
async fn delete_endpoint(world: &mut WorkflowWorld, path: String) {
    if !api_available().await {
        return;
    }
    let path = expand_run_id(&path, &world.run_id);
    let url = format!("{}{}", world.base_url, path);
    let resp = reqwest::Client::new()
        .delete(&url)
        .send()
        .await
        .expect("HTTP DELETE failed");
    world.last_status = resp.status().as_u16();
    world.last_body = resp.text().await.unwrap_or_default();
}

#[then(expr = "the response status is {int}")]
async fn check_status(world: &mut WorkflowWorld, expected: u16) {
    if !api_available().await {
        return;
    }
    assert_eq!(
        world.last_status, expected,
        "Expected status {} but got {} (body: {})",
        expected, world.last_status, world.last_body
    );
}

#[then(expr = "the response body contains {string}")]
async fn check_body_contains(world: &mut WorkflowWorld, text: String) {
    if !api_available().await {
        return;
    }
    assert!(
        world.last_body.contains(&text),
        "Expected body to contain {:?} but got: {}",
        text,
        world.last_body
    );
}

#[then(expr = "the response body does not contain {string}")]
async fn check_body_not_contains(world: &mut WorkflowWorld, text: String) {
    if !api_available().await {
        return;
    }
    assert!(
        !world.last_body.contains(&text),
        "Expected body NOT to contain {:?} but got: {}",
        text,
        world.last_body
    );
}

#[then(expr = "I save the run_id")]
async fn save_run_id(world: &mut WorkflowWorld) {
    if !api_available().await {
        return;
    }
    let body: serde_json::Value = serde_json::from_str(&world.last_body)
        .expect("response body is not valid JSON");
    let run_id = body["run_id"]
        .as_str()
        .expect("no run_id in response body")
        .to_string();
    world.saved_run_id = Some(run_id.clone());
    world.run_id = Some(run_id);
}

#[then(expr = "the run_id matches the previously saved run_id")]
async fn check_run_id_matches(world: &mut WorkflowWorld) {
    if !api_available().await {
        return;
    }
    let body: serde_json::Value = serde_json::from_str(&world.last_body)
        .expect("response body is not valid JSON");
    let current_run_id = body["run_id"].as_str().unwrap_or("").to_string();
    let saved = world.saved_run_id.as_deref().unwrap_or("");
    assert_eq!(
        current_run_id, saved,
        "run_id mismatch: {} != {}",
        current_run_id, saved
    );
}

/// Replace `{run_id}` placeholder in path with the current run_id.
pub fn expand_run_id(path: &str, run_id: &Option<String>) -> String {
    if let Some(ref id) = run_id {
        path.replace("{run_id}", id)
    } else {
        path.to_string()
    }
}
