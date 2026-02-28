use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum WfState {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct StepDef {
    pub name: String,
    pub depends_on: Vec<String>,
    pub max_attempts: u32,
    pub base_delay_ms: u64,
}

#[derive(Debug, Clone)]
pub struct StepEntry {
    pub state: StepState,
    pub attempt: u32,
    pub scheduled_at_ms: u64,
    pub output: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StepRun {
    pub name: String,
    pub state: StepState,
    pub attempt: u32,
    pub scheduled_at_ms: u64,
    pub output: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowRun {
    pub run_id: String,
    pub wf_name: String,
    pub state: WfState,
    pub idem_key: Option<String>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowError {
    NotFound,
    AlreadyDefined,
    DuplicateRun,
    InvalidStep,
    StorageError,
    CycleDetected,
}

// ── Backoff (inline) ──────────────────────────────────────────────────────────

fn next_delay_ms(base_delay_ms: u64, attempt: u32) -> u64 {
    const MAX_DELAY_MS: u64 = 60_000;
    let factor = 1u64.checked_shl(attempt.saturating_sub(1)).unwrap_or(u64::MAX);
    base_delay_ms.saturating_mul(factor).min(MAX_DELAY_MS)
}

// ── Store structures ──────────────────────────────────────────────────────────

struct WfDef {
    steps: Vec<StepDef>,
}

struct RunRecord {
    run: WorkflowRun,
    steps: HashMap<String, StepEntry>,
}

// ── In-memory store ───────────────────────────────────────────────────────────

thread_local! {
    static DEFS: RefCell<HashMap<String, WfDef>> = RefCell::new(HashMap::new());
    static RUNS: RefCell<HashMap<String, RunRecord>> = RefCell::new(HashMap::new());
}

fn with_defs<R>(f: impl FnOnce(&mut HashMap<String, WfDef>) -> R) -> R {
    DEFS.with(|d| f(&mut d.borrow_mut()))
}

fn with_runs<R>(f: impl FnOnce(&mut HashMap<String, RunRecord>) -> R) -> R {
    RUNS.with(|r| f(&mut r.borrow_mut()))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Define a workflow.
pub fn define(name: &str, steps: Vec<StepDef>) -> Result<(), WorkflowError> {
    let n = name.trim().to_string();
    if n.is_empty() {
        return Err(WorkflowError::InvalidStep);
    }
    with_defs(|d| {
        if d.contains_key(&n) {
            return Err(WorkflowError::AlreadyDefined);
        }
        d.insert(n, WfDef { steps });
        Ok(())
    })
}

/// Start a new workflow run. Returns the run-id.
pub fn run(wf_name: &str, _input: Vec<u8>, idem_key: Option<&str>, now_ms: u64) -> Result<String, WorkflowError> {
    let wf = wf_name.trim().to_string();
    let run_id = match idem_key {
        Some(k) => format!("wfrun-{wf}-{k}-{now_ms}"),
        None => format!("wfrun-{wf}-{now_ms}"),
    };

    with_defs(|d| {
        let def = d.get(&wf).ok_or(WorkflowError::NotFound)?;
        let steps: HashMap<String, StepEntry> = def
            .steps
            .iter()
            .map(|s| {
                (
                    s.name.clone(),
                    StepEntry {
                        state: StepState::Pending,
                        attempt: 0,
                        scheduled_at_ms: now_ms,
                        output: None,
                        error: None,
                    },
                )
            })
            .collect();

        with_runs(|r| {
            if r.contains_key(&run_id) {
                return Err(WorkflowError::DuplicateRun);
            }
            r.insert(
                run_id.clone(),
                RunRecord {
                    run: WorkflowRun {
                        run_id: run_id.clone(),
                        wf_name: wf.clone(),
                        state: WfState::Running,
                        idem_key: idem_key.map(str::to_string),
                        created_at_ms: now_ms,
                    },
                    steps,
                },
            );
            Ok(run_id.clone())
        })
    })
}

/// Mark a step as succeeded (idempotent).
pub fn step_done(run_id: &str, step: &str, output: Vec<u8>, now_ms: u64) -> Result<(), WorkflowError> {
    with_runs(|r| {
        let rec = r.get_mut(run_id).ok_or(WorkflowError::NotFound)?;
        let entry = rec.steps.get_mut(step).ok_or(WorkflowError::InvalidStep)?;
        if entry.state == StepState::Succeeded {
            return Ok(()); // idempotent
        }
        entry.state = StepState::Succeeded;
        entry.output = Some(output);
        entry.scheduled_at_ms = now_ms;

        // Check if all steps succeeded → run succeeds
        let all_done = rec.steps.values().all(|e| e.state == StepState::Succeeded);
        if all_done {
            rec.run.state = WfState::Succeeded;
        }
        Ok(())
    })
}

/// Mark a step as failed. Schedules retry or fails the whole run.
pub fn step_failed(run_id: &str, step: &str, error: String, now_ms: u64) -> Result<(), WorkflowError> {
    let wf_name = with_runs(|r| {
        let rec = r.get(run_id).ok_or(WorkflowError::NotFound)?;
        Ok(rec.run.wf_name.clone())
    })?;

    let max_attempts = with_defs(|d| {
        d.get(&wf_name)
            .and_then(|def| def.steps.iter().find(|s| s.name == step))
            .map(|s| (s.max_attempts, s.base_delay_ms))
            .ok_or(WorkflowError::InvalidStep)
    })?;

    with_runs(|r| {
        let rec = r.get_mut(run_id).ok_or(WorkflowError::NotFound)?;
        let entry = rec.steps.get_mut(step).ok_or(WorkflowError::InvalidStep)?;

        entry.attempt += 1;
        entry.error = Some(error);

        if entry.attempt >= max_attempts.0 {
            entry.state = StepState::Failed;
            rec.run.state = WfState::Failed;
        } else {
            let delay = next_delay_ms(max_attempts.1, entry.attempt);
            entry.state = StepState::Pending;
            entry.scheduled_at_ms = now_ms + delay;
        }
        Ok(())
    })
}

/// Return steps whose all depends_on steps are Succeeded and that are Pending and due.
pub fn ready_steps(run_id: &str, now_ms: u64) -> Result<Vec<StepRun>, WorkflowError> {
    let wf_name = with_runs(|r| {
        r.get(run_id).map(|rec| rec.run.wf_name.clone()).ok_or(WorkflowError::NotFound)
    })?;

    with_defs(|d| {
        let def = d.get(&wf_name).ok_or(WorkflowError::NotFound)?;

        with_runs(|r| {
            let rec = r.get(run_id).ok_or(WorkflowError::NotFound)?;

            let mut ready = Vec::new();
            for step_def in &def.steps {
                let entry = rec.steps.get(&step_def.name).ok_or(WorkflowError::InvalidStep)?;
                if entry.state != StepState::Pending {
                    continue;
                }
                if entry.scheduled_at_ms > now_ms {
                    continue;
                }
                let deps_ok = step_def.depends_on.iter().all(|dep| {
                    rec.steps.get(dep).map_or(false, |e| e.state == StepState::Succeeded)
                });
                if deps_ok {
                    ready.push(StepRun {
                        name: step_def.name.clone(),
                        state: entry.state.clone(),
                        attempt: entry.attempt,
                        scheduled_at_ms: entry.scheduled_at_ms,
                        output: entry.output.clone(),
                        error: entry.error.clone(),
                    });
                }
            }
            Ok(ready)
        })
    })
}

/// Retrieve the memoised output of a completed step.
pub fn step_output(run_id: &str, step: &str) -> Result<Option<Vec<u8>>, WorkflowError> {
    with_runs(|r| {
        let rec = r.get(run_id).ok_or(WorkflowError::NotFound)?;
        let entry = rec.steps.get(step).ok_or(WorkflowError::InvalidStep)?;
        Ok(entry.output.clone())
    })
}

/// Get current run state.
pub fn get_run(run_id: &str) -> Result<WorkflowRun, WorkflowError> {
    with_runs(|r| r.get(run_id).map(|rec| rec.run.clone()).ok_or(WorkflowError::NotFound))
}

/// Cancel a running workflow.
pub fn cancel_run(run_id: &str) -> Result<(), WorkflowError> {
    with_runs(|r| {
        let rec = r.get_mut(run_id).ok_or(WorkflowError::NotFound)?;
        match rec.run.state {
            WfState::Running => {
                rec.run.state = WfState::Cancelled;
                for entry in rec.steps.values_mut() {
                    if entry.state == StepState::Pending || entry.state == StepState::Running {
                        entry.state = StepState::Cancelled;
                    }
                }
                Ok(())
            }
            _ => Err(WorkflowError::NotFound),
        }
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_linear_chain() {
        run_test(|| {
            define("chain", vec![
                StepDef { name: "a".into(), depends_on: vec![], max_attempts: 3, base_delay_ms: 100 },
                StepDef { name: "b".into(), depends_on: vec!["a".into()], max_attempts: 3, base_delay_ms: 100 },
                StepDef { name: "c".into(), depends_on: vec!["b".into()], max_attempts: 3, base_delay_ms: 100 },
            ]).unwrap();

            let run_id = run("chain", b"input".to_vec(), None, 0).unwrap();

            // Only "a" is ready (no deps)
            let ready = ready_steps(&run_id, 0).unwrap();
            assert_eq!(ready.len(), 1);
            assert_eq!(ready[0].name, "a");

            step_done(&run_id, "a", b"out-a".to_vec(), 0).unwrap();
            let ready = ready_steps(&run_id, 0).unwrap();
            assert_eq!(ready[0].name, "b");

            step_done(&run_id, "b", b"out-b".to_vec(), 0).unwrap();
            let ready = ready_steps(&run_id, 0).unwrap();
            assert_eq!(ready[0].name, "c");

            step_done(&run_id, "c", b"out-c".to_vec(), 0).unwrap();
            assert_eq!(get_run(&run_id).unwrap().state, WfState::Succeeded);
        });
    }

    #[test]
    fn test_fan_out() {
        run_test(|| {
            define("fan", vec![
                StepDef { name: "root".into(), depends_on: vec![], max_attempts: 1, base_delay_ms: 100 },
                StepDef { name: "left".into(), depends_on: vec!["root".into()], max_attempts: 1, base_delay_ms: 100 },
                StepDef { name: "right".into(), depends_on: vec!["root".into()], max_attempts: 1, base_delay_ms: 100 },
                StepDef { name: "merge".into(), depends_on: vec!["left".into(), "right".into()], max_attempts: 1, base_delay_ms: 100 },
            ]).unwrap();

            let run_id = run("fan", b"".to_vec(), None, 0).unwrap();
            step_done(&run_id, "root", b"r".to_vec(), 0).unwrap();

            let ready = ready_steps(&run_id, 0).unwrap();
            assert_eq!(ready.len(), 2);

            step_done(&run_id, "left", b"l".to_vec(), 0).unwrap();
            step_done(&run_id, "right", b"r".to_vec(), 0).unwrap();

            let ready = ready_steps(&run_id, 0).unwrap();
            assert_eq!(ready[0].name, "merge");

            step_done(&run_id, "merge", b"m".to_vec(), 0).unwrap();
            assert_eq!(get_run(&run_id).unwrap().state, WfState::Succeeded);
        });
    }

    #[test]
    fn test_step_failure_retry() {
        run_test(|| {
            define("retry-wf", vec![
                StepDef { name: "s1".into(), depends_on: vec![], max_attempts: 3, base_delay_ms: 100 },
            ]).unwrap();
            let run_id = run("retry-wf", b"".to_vec(), None, 0).unwrap();
            step_failed(&run_id, "s1", "err".to_string(), 1000).unwrap();

            // attempt=1 < max_attempts=3 → rescheduled
            with_runs(|r| {
                let rec = r.get(&run_id).unwrap();
                let e = rec.steps.get("s1").unwrap();
                assert_eq!(e.state, StepState::Pending);
                assert!(e.scheduled_at_ms > 1000);
            });
        });
    }

    #[test]
    fn test_step_exhaustion_fails_run() {
        run_test(|| {
            define("exhaust-wf", vec![
                StepDef { name: "s1".into(), depends_on: vec![], max_attempts: 2, base_delay_ms: 100 },
            ]).unwrap();
            let run_id = run("exhaust-wf", b"".to_vec(), None, 0).unwrap();
            step_failed(&run_id, "s1", "e1".to_string(), 0).unwrap();
            step_failed(&run_id, "s1", "e2".to_string(), 0).unwrap();
            assert_eq!(get_run(&run_id).unwrap().state, WfState::Failed);
        });
    }

    #[test]
    fn test_cancel_mid_run() {
        run_test(|| {
            define("cancel-wf", vec![
                StepDef { name: "s1".into(), depends_on: vec![], max_attempts: 1, base_delay_ms: 100 },
            ]).unwrap();
            let run_id = run("cancel-wf", b"".to_vec(), None, 0).unwrap();
            cancel_run(&run_id).unwrap();
            assert_eq!(get_run(&run_id).unwrap().state, WfState::Cancelled);
        });
    }

    #[test]
    fn test_memoisation_idempotent() {
        run_test(|| {
            define("memo-wf", vec![
                StepDef { name: "s1".into(), depends_on: vec![], max_attempts: 1, base_delay_ms: 100 },
            ]).unwrap();
            let run_id = run("memo-wf", b"".to_vec(), None, 0).unwrap();
            step_done(&run_id, "s1", b"first".to_vec(), 0).unwrap();
            step_done(&run_id, "s1", b"second".to_vec(), 0).unwrap(); // no-op
            let out = step_output(&run_id, "s1").unwrap().unwrap();
            assert_eq!(out, b"first"); // memoised
        });
    }

    #[test]
    fn test_step_output() {
        run_test(|| {
            define("output-wf", vec![
                StepDef { name: "s1".into(), depends_on: vec![], max_attempts: 1, base_delay_ms: 100 },
            ]).unwrap();
            let run_id = run("output-wf", b"".to_vec(), None, 0).unwrap();
            step_done(&run_id, "s1", b"my-output".to_vec(), 0).unwrap();
            let out = step_output(&run_id, "s1").unwrap().unwrap();
            assert_eq!(out, b"my-output");
        });
    }

    #[test]
    fn test_not_found() {
        run_test(|| {
            assert_eq!(get_run("no-such-run").unwrap_err(), WorkflowError::NotFound);
        });
    }

    #[test]
    fn test_duplicate_run() {
        run_test(|| {
            define("dup-wf", vec![
                StepDef { name: "s".into(), depends_on: vec![], max_attempts: 1, base_delay_ms: 100 },
            ]).unwrap();
            run("dup-wf", b"".to_vec(), Some("k1"), 500).unwrap();
            assert_eq!(
                run("dup-wf", b"".to_vec(), Some("k1"), 500).unwrap_err(),
                WorkflowError::DuplicateRun
            );
        });
    }

    #[test]
    fn test_already_defined() {
        run_test(|| {
            define("defined-wf", vec![
                StepDef { name: "s".into(), depends_on: vec![], max_attempts: 1, base_delay_ms: 100 },
            ]).unwrap();
            assert_eq!(
                define("defined-wf", vec![]).unwrap_err(),
                WorkflowError::AlreadyDefined
            );
        });
    }

    #[test]
    fn test_scheduled_step_not_ready_until_due() {
        run_test(|| {
            define("sched-wf", vec![
                StepDef { name: "s1".into(), depends_on: vec![], max_attempts: 3, base_delay_ms: 1000 },
            ]).unwrap();
            let run_id = run("sched-wf", b"".to_vec(), None, 0).unwrap();
            step_failed(&run_id, "s1", "fail".to_string(), 0).unwrap();
            // scheduled_at_ms = 0 + 1000 = 1000
            let ready = ready_steps(&run_id, 500).unwrap();
            assert!(ready.is_empty());
            let ready = ready_steps(&run_id, 1000).unwrap();
            assert_eq!(ready.len(), 1);
        });
    }
}
