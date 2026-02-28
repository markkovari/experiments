mod steps {
    pub mod common;
    pub mod definition_steps;
    pub mod execution_steps;
    pub mod step_output_steps;
    pub mod sub_workflow_steps;
    pub mod branching_steps;
    pub mod events_steps;
    pub mod yaml_steps;
}

mod hooks;

use cucumber::World;
use tokio::sync::OnceCell;

/// Cached result of one-time HTTP connectivity probe.
/// `true`  = workflow-api is reachable → run live scenarios
/// `false` = not reachable → scenarios will be marked pending (skipped)
static API_UP: OnceCell<bool> = OnceCell::const_new();

/// Check once whether the workflow-api HTTP endpoint is reachable.
pub async fn api_available() -> bool {
    *API_UP
        .get_or_init(|| async {
            let up = reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(800))
                .build()
                .unwrap()
                .get("http://localhost:8080/workflows")
                .send()
                .await
                .is_ok();
            if up {
                println!(
                    "\n[cucumber setup] ✓ workflow-api reachable at http://localhost:8080 \
                     — live scenarios will run"
                );
            } else {
                println!(
                    "\n[cucumber setup] ✗ workflow-api not reachable at http://localhost:8080 \
                     — live scenarios will be skipped\n\
                     \x20             Run `wash up -d && wash app deploy wadm/workflow-api.yaml` to fix this"
                );
            }
            up
        })
        .await
}

#[derive(Debug, Default, World)]
pub struct WorkflowWorld {
    pub base_url: String,
    pub last_status: u16,
    pub last_body: String,
    pub run_id: Option<String>,
    pub saved_run_id: Option<String>,
}

#[tokio::main]
async fn main() {
    // Global pre-hooks: build and deploy the component before any scenario runs.
    hooks::ensure_deployed().await;

    // Probe the API — result cached for all subsequent steps.
    api_available().await;

    // Feature files live at <manifest_dir>/features/
    let features_dir = format!("{}/features", env!("CARGO_MANIFEST_DIR"));

    WorkflowWorld::run(features_dir).await;
}
