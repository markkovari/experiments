//! Global pre-test hooks: build the workflow-api WASM component and deploy it
//! via `wash` before any scenario runs.
//!
//! Each function runs **once** per test process using `tokio::sync::OnceCell`.
//! If NATS/wasmCloud is unreachable the setup is skipped with a warning;
//! scenarios that need a live host will then fail at the HTTP level.

use std::process::Command;
use tokio::sync::OnceCell;

static SETUP_DONE: OnceCell<()> = OnceCell::const_new();

/// Run the global setup exactly once:
///   1. `wash build -p workflow-api`       — compile the WASM component
///   2. `wash app deploy wadm/workflow-api.yaml` — deploy via wadm
///
/// Both steps are best-effort: if `wash` is not installed or wasmCloud is not
/// running the test suite still starts, but live scenarios will fail.
pub async fn ensure_deployed() {
    SETUP_DONE
        .get_or_init(|| async {
            build_component();
            deploy_component();
        })
        .await;
}

/// Build the workflow-api wasmCloud component.
fn build_component() {
    println!("\n[cucumber setup] Building workflow-api component (wash build -p workflow-api)…");

    let status = Command::new("wash")
        .args(["build", "-p", "workflow-api"])
        .current_dir(workspace_root())
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("[cucumber setup] ✓ workflow-api component built successfully");
        }
        Ok(s) => {
            eprintln!(
                "[cucumber setup] ✗ wash build exited with status {}. \
                 Scenarios may fail if the component binary is stale.",
                s
            );
        }
        Err(e) => {
            eprintln!(
                "[cucumber setup] ✗ wash not found or failed to execute: {}. \
                 Install wash (https://wasmcloud.com/docs/installation) and ensure \
                 it is on PATH.",
                e
            );
        }
    }
}

/// Deploy the workflow-api application via wadm.
fn deploy_component() {
    let wadm_manifest = workspace_root().join("wadm").join("workflow-api.yaml");

    if !wadm_manifest.exists() {
        eprintln!(
            "[cucumber setup] ✗ wadm manifest not found at {}. Skipping deploy.",
            wadm_manifest.display()
        );
        return;
    }

    println!(
        "[cucumber setup] Deploying workflow-api (wash app deploy {})…",
        wadm_manifest.display()
    );

    let status = Command::new("wash")
        .args(["app", "deploy", wadm_manifest.to_str().unwrap()])
        .current_dir(workspace_root())
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("[cucumber setup] ✓ workflow-api deployed successfully");
            // Give wasmCloud a moment to start the component.
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        Ok(s) => {
            eprintln!(
                "[cucumber setup] ✗ wash app deploy exited with status {}. \
                 The component may already be deployed (that is OK).",
                s
            );
        }
        Err(e) => {
            eprintln!(
                "[cucumber setup] ✗ Failed to run wash app deploy: {}. \
                 Ensure wasmCloud is running (`wash up -d`).",
                e
            );
        }
    }
}

/// Resolve the workspace root from the test binary's manifest directory.
fn workspace_root() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR points to tests/cucumber/ at compile time.
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // tests/
        .unwrap()
        .parent() // workspace root
        .unwrap()
        .to_path_buf()
}
