// WIT-based cron component.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "cron-component",
    path: "../../wit/wasmcloud-cron",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use cron_core::{
    deregister as core_deregister, due_tasks as core_due_tasks, get_task as core_get_task,
    is_due as core_is_due, list_tasks as core_list_tasks, parse as core_parse,
    register as core_register, set_enabled as core_set_enabled, tick as core_tick,
    CronError as CoreError,
};

#[allow(dead_code)]
fn now_ms() -> u64 { 0 }

#[cfg(target_arch = "wasm32")]
fn core_err(e: CoreError) -> wasmcloud::cron::types::CronError {
    use wasmcloud::cron::types::CronError;
    match e {
        CoreError::InvalidExpression => CronError::InvalidExpression,
        CoreError::NotFound          => CronError::NotFound,
        CoreError::DuplicateTask     => CronError::DuplicateTask,
        CoreError::InvalidName       => CronError::InvalidName,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_schedule(s: cron_core::Schedule) -> wasmcloud::cron::types::Schedule {
    wasmcloud::cron::types::Schedule {
        minutes:       s.minutes,
        hours:         s.hours,
        days_of_month: s.days_of_month,
        months:        s.months,
        days_of_week:  s.days_of_week,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_task(t: cron_core::Task) -> wasmcloud::cron::types::Task {
    wasmcloud::cron::types::Task {
        name:        t.name,
        expression:  t.expression,
        enabled:     t.enabled,
        last_run_ms: t.last_run_ms,
        next_run_ms: t.next_run_ms,
        run_count:   t.run_count,
    }
}

#[cfg(target_arch = "wasm32")]
struct CronComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::cron::cron_api::Guest for CronComponent {
    fn register(name: String, expression: String) -> Result<(), wasmcloud::cron::types::CronError> {
        core_register(&name, &expression, now_ms()).map_err(core_err)
    }
    fn parse(expression: String) -> Result<wasmcloud::cron::types::Schedule, wasmcloud::cron::types::CronError> {
        core_parse(&expression).map(wit_schedule).map_err(core_err)
    }
    fn is_due(name: String, now_ms: u64) -> Result<bool, wasmcloud::cron::types::CronError> {
        core_is_due(&name, now_ms).map_err(core_err)
    }
    fn tick(name: String, now_ms: u64) -> Result<(), wasmcloud::cron::types::CronError> {
        core_tick(&name, now_ms).map_err(core_err)
    }
    fn due_tasks(now_ms: u64) -> Result<Vec<wasmcloud::cron::types::Task>, wasmcloud::cron::types::CronError> {
        core_due_tasks(now_ms).map(|v| v.into_iter().map(wit_task).collect()).map_err(core_err)
    }
    fn get_task(name: String) -> Result<wasmcloud::cron::types::Task, wasmcloud::cron::types::CronError> {
        core_get_task(&name).map(wit_task).map_err(core_err)
    }
    fn list_tasks() -> Result<Vec<wasmcloud::cron::types::Task>, wasmcloud::cron::types::CronError> {
        core_list_tasks().map(|v| v.into_iter().map(wit_task).collect()).map_err(core_err)
    }
    fn set_enabled(name: String, enabled: bool) -> Result<(), wasmcloud::cron::types::CronError> {
        core_set_enabled(&name, enabled).map_err(core_err)
    }
    fn deregister(name: String) -> Result<(), wasmcloud::cron::types::CronError> {
        core_deregister(&name).map_err(core_err)
    }
}

#[cfg(target_arch = "wasm32")]
export!(CronComponent);

// ── native re-exports ─────────────────────────────────────────────────────────

pub use cron_core::{
    deregister, due_tasks, get_task, is_due, list_tasks, parse, register, set_enabled, tick,
    CronError, Schedule, Task,
};

#[cfg(test)]
mod tests {
    use super::*;

    const T0: u64 = 1_705_314_600_000; // 2024-01-15 10:30:00 UTC

    #[test]
    fn roundtrip_register_tick() {
        std::thread::spawn(|| {
            register("comp-job", "*/5 * * * *", T0).unwrap();
            let t = get_task("comp-job").unwrap();
            let next = t.next_run_ms.unwrap();
            tick("comp-job", next).unwrap();
            let after = get_task("comp-job").unwrap();
            assert_eq!(after.run_count, 1);
            assert!(after.next_run_ms.unwrap() > next);
        }).join().unwrap();
    }

    #[test]
    fn disabled_task_not_due() {
        std::thread::spawn(|| {
            register("comp-disabled", "* * * * *", T0).unwrap();
            set_enabled("comp-disabled", false).unwrap();
            let t = get_task("comp-disabled").unwrap();
            let next = t.next_run_ms.unwrap();
            assert!(!is_due("comp-disabled", next + 600_000).unwrap());
        }).join().unwrap();
    }
}
