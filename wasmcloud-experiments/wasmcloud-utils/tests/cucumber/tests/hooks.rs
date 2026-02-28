//! Global pre-test hooks: build the workflow-api WASM component and deploy it
//! via `wash` before any scenario runs.
//!
//! Each function runs **once** per test process using `tokio::sync::OnceCell`.
//! If wasmCloud is unreachable the setup is skipped with a warning; scenarios
//! that need a live host will then fail at the HTTP level.

use std::process::Command;
use tokio::sync::OnceCell;

static SETUP_DONE: OnceCell<()> = OnceCell::const_new();

/// Run the global setup exactly once:
///   1. `cargo build -p workflow-api --target wasm32-wasip2 --release`
///      — compile the WASM component artifact
///   2. `wash app deploy wadm/workflow-api.yaml`
///      — deploy via wadm (best-effort; already-deployed is not an error)
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

/// Build the workflow-api wasmCloud component as a WASM binary.
fn build_component() {
    println!(
        "\n[cucumber setup] Building workflow-api WASM component \
         (cargo build -p workflow-api --target wasm32-wasip2 --release)…"
    );

    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "workflow-api",
            "--target",
            "wasm32-wasip2",
            "--release",
        ])
        .current_dir(workspace_root())
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("[cucumber setup] ✓ workflow-api WASM component built successfully");
        }
        Ok(s) => {
            eprintln!(
                "[cucumber setup] ✗ cargo build exited with status {}. \
                 Ensure wasm32-wasip2 target is installed: \
                 `rustup target add wasm32-wasip2`",
                s
            );
        }
        Err(e) => {
            eprintln!(
                "[cucumber setup] ✗ Failed to run cargo build: {}",
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
/// CARGO_MANIFEST_DIR points to `tests/cucumber/` at compile time.
pub fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // tests/
        .unwrap()
        .parent() // workspace root
        .unwrap()
        .to_path_buf()
}
