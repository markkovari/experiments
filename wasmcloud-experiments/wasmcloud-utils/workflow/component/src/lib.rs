// WIT-based workflow component.
// Targets the `workflow-component` world defined in wit/wasmcloud-workflow/workflow.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "workflow-component",
    path: "../../wit/wasmcloud-workflow",
    generate_all,
});

use workflow_core::{
    cancel_run as core_cancel_run, define as core_define, get_run as core_get_run,
    ready_steps as core_ready_steps, run as core_run, step_done as core_step_done,
    step_failed as core_step_failed, step_output as core_step_output,
    StepDef as CoreStepDef, StepState as CoreStepState, WfState as CoreWfState,
    WorkflowError as CoreError,
};

// ---- type conversions (wasm32 only) -----------------------------------------

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::workflow::types::WorkflowError {
    use wasmcloud::workflow::types::WorkflowError;
    match e {
        CoreError::NotFound => WorkflowError::NotFound,
        CoreError::AlreadyDefined => WorkflowError::AlreadyDefined,
        CoreError::DuplicateRun => WorkflowError::DuplicateRun,
        CoreError::InvalidStep => WorkflowError::InvalidStep,
        CoreError::StorageError => WorkflowError::StorageError,
        CoreError::CycleDetected => WorkflowError::CycleDetected,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_wf_state(s: CoreWfState) -> wasmcloud::workflow::types::WfState {
    use wasmcloud::workflow::types::WfState;
    match s {
        CoreWfState::Running => WfState::Running,
        CoreWfState::Succeeded => WfState::Succeeded,
        CoreWfState::Failed => WfState::Failed,
        CoreWfState::Cancelled => WfState::Cancelled,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_step_state(s: CoreStepState) -> wasmcloud::workflow::types::StepState {
    use wasmcloud::workflow::types::StepState;
    match s {
        CoreStepState::Pending => StepState::Pending,
        CoreStepState::Running => StepState::Running,
        CoreStepState::Succeeded => StepState::Succeeded,
        CoreStepState::Failed => StepState::Failed,
        CoreStepState::Cancelled => StepState::Cancelled,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_step_run(s: workflow_core::StepRun) -> wasmcloud::workflow::types::StepRun {
    wasmcloud::workflow::types::StepRun {
        name: s.name,
        state: wit_step_state(s.state),
        attempt: s.attempt,
        scheduled_at_ms: s.scheduled_at_ms,
        output: s.output,
        error: s.error,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_workflow_run(r: workflow_core::WorkflowRun) -> wasmcloud::workflow::types::WorkflowRun {
    wasmcloud::workflow::types::WorkflowRun {
        run_id: r.run_id,
        wf_name: r.wf_name,
        state: wit_wf_state(r.state),
        idem_key: r.idem_key,
        created_at_ms: r.created_at_ms,
    }
}

// ---- WIT guest implementation -----------------------------------------------

#[cfg(target_arch = "wasm32")]
struct WorkflowComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::workflow::workflow_api::Guest for WorkflowComponent {
    fn define(
        name: String,
        steps: Vec<wasmcloud::workflow::types::StepDef>,
    ) -> Result<(), wasmcloud::workflow::types::WorkflowError> {
        let core_steps = steps
            .into_iter()
            .map(|s| CoreStepDef {
                name: s.name,
                depends_on: s.depends_on,
                max_attempts: s.max_attempts,
                base_delay_ms: s.base_delay_ms,
            })
            .collect();
        core_define(&name, core_steps).map_err(core_error)
    }

    fn run(
        wf_name: String,
        input: Vec<u8>,
        idem_key: Option<String>,
    ) -> Result<String, wasmcloud::workflow::types::WorkflowError> {
        core_run(&wf_name, input, idem_key.as_deref(), 0).map_err(core_error)
    }

    fn step_done(
        run_id: String,
        step: String,
        output: Vec<u8>,
        now_ms: u64,
    ) -> Result<(), wasmcloud::workflow::types::WorkflowError> {
        core_step_done(&run_id, &step, output, now_ms).map_err(core_error)
    }

    fn step_failed(
        run_id: String,
        step: String,
        error: String,
        now_ms: u64,
    ) -> Result<(), wasmcloud::workflow::types::WorkflowError> {
        core_step_failed(&run_id, &step, error, now_ms).map_err(core_error)
    }

    fn ready_steps(
        run_id: String,
        now_ms: u64,
    ) -> Result<Vec<wasmcloud::workflow::types::StepRun>, wasmcloud::workflow::types::WorkflowError>
    {
        core_ready_steps(&run_id, now_ms)
            .map(|v| v.into_iter().map(wit_step_run).collect())
            .map_err(core_error)
    }

    fn step_output(
        run_id: String,
        step: String,
    ) -> Result<Option<Vec<u8>>, wasmcloud::workflow::types::WorkflowError> {
        core_step_output(&run_id, &step).map_err(core_error)
    }

    fn get_run(
        run_id: String,
    ) -> Result<wasmcloud::workflow::types::WorkflowRun, wasmcloud::workflow::types::WorkflowError>
    {
        core_get_run(&run_id).map(wit_workflow_run).map_err(core_error)
    }

    fn cancel_run(run_id: String) -> Result<(), wasmcloud::workflow::types::WorkflowError> {
        core_cancel_run(&run_id).map_err(core_error)
    }

    fn get_secret(name: String) -> Result<Vec<u8>, String> {
        use wasmcloud::secrets::secret_store::{get, SecretError};
        get(&name)
            .map(|v| v.data)
            .map_err(|e| match e {
                SecretError::NotFound => format!("secret not found: {name}"),
                SecretError::NotInitialized => "secret store not initialized".to_string(),
                SecretError::EncryptionError => "decryption failed".to_string(),
                SecretError::StorageError => "storage error".to_string(),
                SecretError::PermissionDenied => "permission denied".to_string(),
                SecretError::InvalidConfig => "invalid config".to_string(),
                SecretError::AlreadyExists => "already exists".to_string(),
            })
    }
}

#[cfg(target_arch = "wasm32")]
export!(WorkflowComponent);

// ---- native helpers (cargo check / tests) -----------------------------------

pub fn wf_define(name: &str, steps: Vec<CoreStepDef>) -> Result<(), CoreError> {
    core_define(name, steps)
}

pub fn wf_run(wf_name: &str, input: Vec<u8>, idem_key: Option<&str>, now_ms: u64) -> Result<String, CoreError> {
    core_run(wf_name, input, idem_key, now_ms)
}

pub fn wf_step_done(run_id: &str, step: &str, output: Vec<u8>, now_ms: u64) -> Result<(), CoreError> {
    core_step_done(run_id, step, output, now_ms)
}

pub fn wf_get_run(run_id: &str) -> Result<workflow_core::WorkflowRun, CoreError> {
    core_get_run(run_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_define_and_run() {
        wf_define("comp-wf", vec![
            CoreStepDef {
                name: "step1".into(),
                depends_on: vec![],
                max_attempts: 1,
                base_delay_ms: 100,
            },
        ]).unwrap();
        let run_id = wf_run("comp-wf", b"data".to_vec(), None, 0).unwrap();
        let run = wf_get_run(&run_id).unwrap();
        assert_eq!(run.wf_name, "comp-wf");
        assert_eq!(run.state, workflow_core::WfState::Running);
    }

    #[test]
    fn roundtrip_step_done() {
        wf_define("comp-wf2", vec![
            CoreStepDef {
                name: "s1".into(),
                depends_on: vec![],
                max_attempts: 1,
                base_delay_ms: 100,
            },
        ]).unwrap();
        let run_id = wf_run("comp-wf2", b"".to_vec(), None, 0).unwrap();
        wf_step_done(&run_id, "s1", b"out".to_vec(), 0).unwrap();
        let run = wf_get_run(&run_id).unwrap();
        assert_eq!(run.state, workflow_core::WfState::Succeeded);
    }
}
