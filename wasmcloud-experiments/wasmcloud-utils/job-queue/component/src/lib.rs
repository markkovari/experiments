// WIT-based job-queue component.
// Targets the `job-queue-component` world defined in wit/wasmcloud-job-queue/job-queue.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "job-queue-component",
    path: "../../wit/wasmcloud-job-queue",
    generate_all,
});

use job_queue_core::{
    cancel as core_cancel, dead_letters as core_dead_letters, due as core_due, enqueue as core_enqueue,
    fail as core_fail, get as core_get, pause as core_pause, resume as core_resume, start as core_start,
    succeed as core_succeed, JobError as CoreError, RunState as CoreState,
};

// ---- type conversions (wasm32 only) -----------------------------------------

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::job_queue::types::JobError {
    use wasmcloud::job_queue::types::JobError;
    match e {
        CoreError::NotFound => JobError::NotFound,
        CoreError::InvalidFn => JobError::InvalidFn,
        CoreError::AlreadyDone => JobError::AlreadyDone,
        CoreError::StorageError => JobError::StorageError,
        CoreError::DispatchError => JobError::DispatchError,
        CoreError::DuplicateRun => JobError::DuplicateRun,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_state(s: CoreState) -> wasmcloud::job_queue::types::RunState {
    use wasmcloud::job_queue::types::RunState;
    match s {
        CoreState::Pending => RunState::Pending,
        CoreState::Running => RunState::Running,
        CoreState::Succeeded => RunState::Succeeded,
        CoreState::Failed => RunState::Failed,
        CoreState::Cancelled => RunState::Cancelled,
        CoreState::Paused => RunState::Paused,
        CoreState::DeadLetter => RunState::DeadLetter,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_run_info(info: job_queue_core::RunInfo) -> wasmcloud::job_queue::types::RunInfo {
    wasmcloud::job_queue::types::RunInfo {
        run_id: info.run_id,
        fn_name: info.fn_name,
        state: wit_state(info.state),
        attempt: info.attempt,
        max_attempts: info.max_attempts,
        scheduled_at_ms: info.scheduled_at_ms,
        idem_key: info.idem_key,
        output: info.output,
        error: info.error,
    }
}

// ---- WIT guest implementation -----------------------------------------------

#[cfg(target_arch = "wasm32")]
struct JobQueueComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::job_queue::job_api::Guest for JobQueueComponent {
    fn enqueue(
        fn_name: String,
        payload: Vec<u8>,
        idem_key: Option<String>,
        max_attempts: u32,
        base_delay_ms: u64,
    ) -> Result<String, wasmcloud::job_queue::types::JobError> {
        core_enqueue(&fn_name, payload, idem_key.as_deref(), max_attempts, base_delay_ms, 0)
            .map_err(core_error)
    }

    fn start(
        run_id: String,
        now_ms: u64,
    ) -> Result<(), wasmcloud::job_queue::types::JobError> {
        core_start(&run_id, now_ms).map_err(core_error)
    }

    fn succeed(
        run_id: String,
        output: Vec<u8>,
        now_ms: u64,
    ) -> Result<(), wasmcloud::job_queue::types::JobError> {
        core_succeed(&run_id, output, now_ms).map_err(core_error)
    }

    fn fail(
        run_id: String,
        error: String,
        now_ms: u64,
    ) -> Result<(), wasmcloud::job_queue::types::JobError> {
        core_fail(&run_id, error, now_ms).map_err(core_error)
    }

    fn cancel(run_id: String) -> Result<(), wasmcloud::job_queue::types::JobError> {
        core_cancel(&run_id).map_err(core_error)
    }

    fn pause(run_id: String) -> Result<(), wasmcloud::job_queue::types::JobError> {
        core_pause(&run_id).map_err(core_error)
    }

    fn resume(
        run_id: String,
        now_ms: u64,
    ) -> Result<(), wasmcloud::job_queue::types::JobError> {
        core_resume(&run_id, now_ms).map_err(core_error)
    }

    fn get(
        run_id: String,
    ) -> Result<wasmcloud::job_queue::types::RunInfo, wasmcloud::job_queue::types::JobError> {
        core_get(&run_id).map(wit_run_info).map_err(core_error)
    }

    fn due(
        now_ms: u64,
    ) -> Result<Vec<String>, wasmcloud::job_queue::types::JobError> {
        core_due(now_ms).map_err(core_error)
    }

    fn dead_letters() -> Result<Vec<String>, wasmcloud::job_queue::types::JobError> {
        core_dead_letters().map_err(core_error)
    }
}

#[cfg(target_arch = "wasm32")]
export!(JobQueueComponent);

// ---- native helpers (cargo check / tests) -----------------------------------

pub fn jq_enqueue(
    fn_name: &str,
    payload: Vec<u8>,
    idem_key: Option<&str>,
    max_attempts: u32,
    base_delay_ms: u64,
    now_ms: u64,
) -> Result<String, CoreError> {
    core_enqueue(fn_name, payload, idem_key, max_attempts, base_delay_ms, now_ms)
}

pub fn jq_start(run_id: &str, now_ms: u64) -> Result<(), CoreError> {
    core_start(run_id, now_ms)
}

pub fn jq_succeed(run_id: &str, output: Vec<u8>, now_ms: u64) -> Result<(), CoreError> {
    core_succeed(run_id, output, now_ms)
}

pub fn jq_fail(run_id: &str, error: String, now_ms: u64) -> Result<(), CoreError> {
    core_fail(run_id, error, now_ms)
}

pub fn jq_get(run_id: &str) -> Result<job_queue_core::RunInfo, CoreError> {
    core_get(run_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_enqueue_succeed() {
        let id = jq_enqueue("comp-fn", b"hello".to_vec(), None, 3, 100, 0).unwrap();
        jq_start(&id, 0).unwrap();
        jq_succeed(&id, b"ok".to_vec(), 0).unwrap();
        let info = jq_get(&id).unwrap();
        assert_eq!(info.state, job_queue_core::RunState::Succeeded);
    }

    #[test]
    fn roundtrip_fail_retry() {
        let id = jq_enqueue("comp-retry-fn", b"".to_vec(), None, 3, 200, 0).unwrap();
        jq_start(&id, 0).unwrap();
        jq_fail(&id, "oops".to_string(), 1000).unwrap();
        let info = jq_get(&id).unwrap();
        assert_eq!(info.state, job_queue_core::RunState::Pending);
        assert!(info.scheduled_at_ms > 1000);
    }
}
