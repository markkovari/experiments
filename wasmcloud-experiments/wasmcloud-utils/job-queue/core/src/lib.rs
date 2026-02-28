use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum RunState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Paused,
    DeadLetter,
}

#[derive(Debug, Clone)]
pub struct RunEntry {
    pub run_id: String,
    pub fn_name: String,
    pub payload: Vec<u8>,
    pub state: RunState,
    pub attempt: u32,
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub scheduled_at_ms: u64,
    pub idem_key: Option<String>,
    pub output: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RunInfo {
    pub run_id: String,
    pub fn_name: String,
    pub state: RunState,
    pub attempt: u32,
    pub max_attempts: u32,
    pub scheduled_at_ms: u64,
    pub idem_key: Option<String>,
    pub output: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobError {
    NotFound,
    InvalidFn,
    AlreadyDone,
    StorageError,
    DispatchError,
    DuplicateRun,
}

// ── Backoff (inline, no circular dep on retry-core) ───────────────────────────

/// Compute exponential backoff: base_delay_ms * 2^(attempt-1), capped at 60_000 ms.
fn next_delay_ms(base_delay_ms: u64, attempt: u32) -> u64 {
    const MAX_DELAY_MS: u64 = 60_000;
    let factor = 1u64.checked_shl(attempt.saturating_sub(1)).unwrap_or(u64::MAX);
    base_delay_ms.saturating_mul(factor).min(MAX_DELAY_MS)
}

// ── Run ID generation ─────────────────────────────────────────────────────────

/// Deterministic run ID from fn_name + idem_key + enqueue_ms.
fn make_run_id(fn_name: &str, idem_key: Option<&str>, enqueue_ms: u64) -> String {
    match idem_key {
        Some(k) => format!("run-{fn_name}-{k}-{enqueue_ms}"),
        None => format!("run-{fn_name}-{enqueue_ms}"),
    }
}

// ── Validation ────────────────────────────────────────────────────────────────

fn validate_fn_name(name: &str) -> Result<String, JobError> {
    let n = name.trim().to_string();
    if n.is_empty() {
        return Err(JobError::InvalidFn);
    }
    Ok(n)
}

// ── In-memory store ───────────────────────────────────────────────────────────

thread_local! {
    static STORE: RefCell<HashMap<String, RunEntry>> = RefCell::new(HashMap::new());
}

fn with_store<R>(f: impl FnOnce(&mut HashMap<String, RunEntry>) -> R) -> R {
    STORE.with(|s| f(&mut s.borrow_mut()))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Enqueue a new job run. Returns the run-id.
pub fn enqueue(
    fn_name: &str,
    payload: Vec<u8>,
    idem_key: Option<&str>,
    max_attempts: u32,
    base_delay_ms: u64,
    now_ms: u64,
) -> Result<String, JobError> {
    let name = validate_fn_name(fn_name)?;
    let run_id = make_run_id(&name, idem_key, now_ms);
    with_store(|m| {
        if m.contains_key(&run_id) {
            return Err(JobError::DuplicateRun);
        }
        m.insert(
            run_id.clone(),
            RunEntry {
                run_id: run_id.clone(),
                fn_name: name,
                payload,
                state: RunState::Pending,
                attempt: 0,
                max_attempts,
                base_delay_ms,
                scheduled_at_ms: now_ms,
                idem_key: idem_key.map(str::to_string),
                output: None,
                error: None,
            },
        );
        Ok(run_id)
    })
}

/// Mark a run as Running (Pending → Running).
pub fn start(run_id: &str, _now_ms: u64) -> Result<(), JobError> {
    with_store(|m| {
        let entry = m.get_mut(run_id).ok_or(JobError::NotFound)?;
        match entry.state {
            RunState::Pending => {
                entry.state = RunState::Running;
                entry.attempt += 1;
                Ok(())
            }
            RunState::Succeeded | RunState::Cancelled | RunState::DeadLetter => {
                Err(JobError::AlreadyDone)
            }
            _ => Ok(()),
        }
    })
}

/// Mark a run as Succeeded.
pub fn succeed(run_id: &str, output: Vec<u8>, _now_ms: u64) -> Result<(), JobError> {
    with_store(|m| {
        let entry = m.get_mut(run_id).ok_or(JobError::NotFound)?;
        match entry.state {
            RunState::Running | RunState::Pending => {
                entry.state = RunState::Succeeded;
                entry.output = Some(output);
                Ok(())
            }
            RunState::Succeeded => Err(JobError::AlreadyDone),
            RunState::Cancelled | RunState::DeadLetter => Err(JobError::AlreadyDone),
            _ => Err(JobError::AlreadyDone),
        }
    })
}

/// Mark a run as Failed. Schedules retry if attempts remain, else DeadLetter.
pub fn fail(run_id: &str, error: String, now_ms: u64) -> Result<(), JobError> {
    with_store(|m| {
        let entry = m.get_mut(run_id).ok_or(JobError::NotFound)?;
        match entry.state {
            RunState::Cancelled | RunState::Succeeded | RunState::DeadLetter => {
                return Err(JobError::AlreadyDone);
            }
            _ => {}
        }
        entry.error = Some(error);
        if entry.attempt >= entry.max_attempts {
            entry.state = RunState::DeadLetter;
        } else {
            let delay = next_delay_ms(entry.base_delay_ms, entry.attempt);
            entry.scheduled_at_ms = now_ms + delay;
            entry.state = RunState::Pending;
        }
        Ok(())
    })
}

/// Cancel a run.
pub fn cancel(run_id: &str) -> Result<(), JobError> {
    with_store(|m| {
        let entry = m.get_mut(run_id).ok_or(JobError::NotFound)?;
        match entry.state {
            RunState::Succeeded | RunState::DeadLetter => Err(JobError::AlreadyDone),
            _ => {
                entry.state = RunState::Cancelled;
                Ok(())
            }
        }
    })
}

/// Pause a running/pending run.
pub fn pause(run_id: &str) -> Result<(), JobError> {
    with_store(|m| {
        let entry = m.get_mut(run_id).ok_or(JobError::NotFound)?;
        match entry.state {
            RunState::Pending | RunState::Running => {
                entry.state = RunState::Paused;
                Ok(())
            }
            _ => Err(JobError::AlreadyDone),
        }
    })
}

/// Resume a paused run.
pub fn resume(run_id: &str, now_ms: u64) -> Result<(), JobError> {
    with_store(|m| {
        let entry = m.get_mut(run_id).ok_or(JobError::NotFound)?;
        match entry.state {
            RunState::Paused => {
                entry.state = RunState::Pending;
                entry.scheduled_at_ms = now_ms;
                Ok(())
            }
            _ => Err(JobError::AlreadyDone),
        }
    })
}

/// Get run info by run-id.
pub fn get(run_id: &str) -> Result<RunInfo, JobError> {
    with_store(|m| {
        m.get(run_id).map(|e| RunInfo {
            run_id: e.run_id.clone(),
            fn_name: e.fn_name.clone(),
            state: e.state.clone(),
            attempt: e.attempt,
            max_attempts: e.max_attempts,
            scheduled_at_ms: e.scheduled_at_ms,
            idem_key: e.idem_key.clone(),
            output: e.output.clone(),
            error: e.error.clone(),
        }).ok_or(JobError::NotFound)
    })
}

/// Return run-ids that are Pending and scheduled_at_ms <= now_ms.
pub fn due(now_ms: u64) -> Result<Vec<String>, JobError> {
    with_store(|m| {
        Ok(m.values()
            .filter(|e| e.state == RunState::Pending && e.scheduled_at_ms <= now_ms)
            .map(|e| e.run_id.clone())
            .collect())
    })
}

/// Return all dead-letter run-ids.
pub fn dead_letters() -> Result<Vec<String>, JobError> {
    with_store(|m| {
        Ok(m.values()
            .filter(|e| e.state == RunState::DeadLetter)
            .map(|e| e.run_id.clone())
            .collect())
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_enqueue_and_get() {
        run(|| {
            let id = enqueue("my-fn", b"data".to_vec(), None, 3, 100, 1000).unwrap();
            let info = get(&id).unwrap();
            assert_eq!(info.fn_name, "my-fn");
            assert_eq!(info.state, RunState::Pending);
            assert_eq!(info.attempt, 0);
        });
    }

    #[test]
    fn test_start_increments_attempt() {
        run(|| {
            let id = enqueue("fn-start", b"".to_vec(), None, 3, 100, 0).unwrap();
            start(&id, 0).unwrap();
            let info = get(&id).unwrap();
            assert_eq!(info.state, RunState::Running);
            assert_eq!(info.attempt, 1);
        });
    }

    #[test]
    fn test_succeed() {
        run(|| {
            let id = enqueue("fn-succeed", b"".to_vec(), None, 3, 100, 0).unwrap();
            start(&id, 0).unwrap();
            succeed(&id, b"result".to_vec(), 0).unwrap();
            let info = get(&id).unwrap();
            assert_eq!(info.state, RunState::Succeeded);
            assert_eq!(info.output, Some(b"result".to_vec()));
        });
    }

    #[test]
    fn test_fail_schedules_retry() {
        run(|| {
            let id = enqueue("fn-retry", b"".to_vec(), None, 3, 100, 0).unwrap();
            start(&id, 0).unwrap();
            fail(&id, "boom".to_string(), 1000).unwrap();
            let info = get(&id).unwrap();
            assert_eq!(info.state, RunState::Pending);
            // attempt=1, delay = 100 * 2^0 = 100 ms
            assert_eq!(info.scheduled_at_ms, 1100);
        });
    }

    #[test]
    fn test_exhausted_becomes_dead_letter() {
        run(|| {
            let id = enqueue("fn-exhaust", b"".to_vec(), None, 2, 100, 0).unwrap();
            start(&id, 0).unwrap();
            fail(&id, "e1".to_string(), 100).unwrap();
            // retry: attempt=1, attempt < max_attempts(2) → Pending
            start(&id, 200).unwrap();
            fail(&id, "e2".to_string(), 300).unwrap();
            // attempt=2 >= max_attempts(2) → DeadLetter
            let info = get(&id).unwrap();
            assert_eq!(info.state, RunState::DeadLetter);
        });
    }

    #[test]
    fn test_dead_letters_list() {
        run(|| {
            let id = enqueue("fn-dl", b"".to_vec(), None, 1, 100, 0).unwrap();
            start(&id, 0).unwrap();
            fail(&id, "gone".to_string(), 0).unwrap();
            let dls = dead_letters().unwrap();
            assert!(dls.contains(&id));
        });
    }

    #[test]
    fn test_cancel() {
        run(|| {
            let id = enqueue("fn-cancel", b"".to_vec(), None, 3, 100, 0).unwrap();
            cancel(&id).unwrap();
            assert_eq!(get(&id).unwrap().state, RunState::Cancelled);
        });
    }

    #[test]
    fn test_pause_and_resume() {
        run(|| {
            let id = enqueue("fn-pause", b"".to_vec(), None, 3, 100, 0).unwrap();
            pause(&id).unwrap();
            assert_eq!(get(&id).unwrap().state, RunState::Paused);
            resume(&id, 5000).unwrap();
            let info = get(&id).unwrap();
            assert_eq!(info.state, RunState::Pending);
            assert_eq!(info.scheduled_at_ms, 5000);
        });
    }

    #[test]
    fn test_due_returns_ready_runs() {
        run(|| {
            let id = enqueue("fn-due", b"".to_vec(), None, 3, 100, 1000).unwrap();
            // not due yet
            assert!(!due(999).unwrap().contains(&id));
            // due at 1000
            assert!(due(1000).unwrap().contains(&id));
        });
    }

    #[test]
    fn test_duplicate_run_id() {
        run(|| {
            enqueue("fn-dup", b"".to_vec(), Some("k1"), 3, 100, 500).unwrap();
            assert_eq!(
                enqueue("fn-dup", b"".to_vec(), Some("k1"), 3, 100, 500).unwrap_err(),
                JobError::DuplicateRun
            );
        });
    }

    #[test]
    fn test_invalid_fn_name() {
        run(|| {
            assert_eq!(
                enqueue("", b"".to_vec(), None, 3, 100, 0).unwrap_err(),
                JobError::InvalidFn
            );
        });
    }

    #[test]
    fn test_backoff_exponential() {
        assert_eq!(next_delay_ms(100, 1), 100);
        assert_eq!(next_delay_ms(100, 2), 200);
        assert_eq!(next_delay_ms(100, 3), 400);
        assert_eq!(next_delay_ms(100, 10), 51_200);
        assert_eq!(next_delay_ms(100, 20), 60_000); // capped
    }

    #[test]
    fn test_not_found() {
        run(|| {
            assert_eq!(get("nonexistent-id").unwrap_err(), JobError::NotFound);
        });
    }
}
