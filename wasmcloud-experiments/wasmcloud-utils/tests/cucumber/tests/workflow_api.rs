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
    // This mirrors the pattern used in tests/e2e (OnceCell for one-time setup).
    hooks::ensure_deployed().await;

    WorkflowWorld::run("tests/cucumber/features").await;
}
