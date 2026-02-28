use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum OverallStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub name: String,
    pub healthy: bool,
    pub last_check_ms: u64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthError {
    NotFound,
    InvalidName,
    DuplicateProbe,
}

#[derive(Debug, Clone)]
struct ProbeEntry {
    name: String,
    healthy: bool,
    last_check_ms: u64,
    message: Option<String>,
}

// ── Thread-local state ────────────────────────────────────────────────────────

thread_local! {
    static PROBES: RefCell<HashMap<String, ProbeEntry>> = RefCell::new(HashMap::new());
}

fn with_probes<R>(f: impl FnOnce(&mut HashMap<String, ProbeEntry>) -> R) -> R {
    PROBES.with(|p| f(&mut p.borrow_mut()))
}

fn validate_name(name: &str) -> Result<String, HealthError> {
    let n = name.trim().to_string();
    if n.is_empty() {
        return Err(HealthError::InvalidName);
    }
    Ok(n)
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Register a new probe. Returns DuplicateProbe if name already exists.
pub fn register(name: &str) -> Result<(), HealthError> {
    let n = validate_name(name)?;
    with_probes(|m| {
        if m.contains_key(&n) {
            return Err(HealthError::DuplicateProbe);
        }
        m.insert(n.clone(), ProbeEntry {
            name: n,
            healthy: true,
            last_check_ms: 0,
            message: None,
        });
        Ok(())
    })
}

/// Record a health result for a named probe.
pub fn record_result(name: &str, healthy: bool, message: Option<String>, now_ms: u64) -> Result<(), HealthError> {
    let n = validate_name(name)?;
    with_probes(|m| {
        let entry = m.get_mut(&n).ok_or(HealthError::NotFound)?;
        entry.healthy = healthy;
        entry.last_check_ms = now_ms;
        entry.message = message;
        Ok(())
    })
}

/// Compute overall status:
/// - All healthy → Healthy
/// - All unhealthy → Unhealthy
/// - Mix → Degraded
/// - No probes → Healthy
pub fn status() -> Result<OverallStatus, HealthError> {
    with_probes(|m| {
        if m.is_empty() {
            return Ok(OverallStatus::Healthy);
        }
        let total = m.len();
        let healthy_count = m.values().filter(|e| e.healthy).count();
        if healthy_count == total {
            Ok(OverallStatus::Healthy)
        } else if healthy_count == 0 {
            Ok(OverallStatus::Unhealthy)
        } else {
            Ok(OverallStatus::Degraded)
        }
    })
}

/// Return all registered probe results.
pub fn all_probes() -> Result<Vec<ProbeResult>, HealthError> {
    Ok(with_probes(|m| {
        m.values().map(|e| ProbeResult {
            name: e.name.clone(),
            healthy: e.healthy,
            last_check_ms: e.last_check_ms,
            message: e.message.clone(),
        }).collect()
    }))
}

/// Deregister a probe by name.
pub fn deregister(name: &str) -> Result<(), HealthError> {
    let n = validate_name(name)?;
    with_probes(|m| {
        m.remove(&n).ok_or(HealthError::NotFound)?;
        Ok(())
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
    fn test_register_and_status_healthy() {
        run(|| {
            register("db").unwrap();
            record_result("db", true, None, 100).unwrap();
            assert_eq!(status().unwrap(), OverallStatus::Healthy);
        });
    }

    #[test]
    fn test_all_unhealthy() {
        run(|| {
            register("svc").unwrap();
            record_result("svc", false, Some("timeout".to_string()), 100).unwrap();
            assert_eq!(status().unwrap(), OverallStatus::Unhealthy);
        });
    }

    #[test]
    fn test_degraded_on_mix() {
        run(|| {
            register("a").unwrap();
            register("b").unwrap();
            record_result("a", true, None, 100).unwrap();
            record_result("b", false, None, 100).unwrap();
            assert_eq!(status().unwrap(), OverallStatus::Degraded);
        });
    }

    #[test]
    fn test_duplicate_probe_error() {
        run(|| {
            register("dup").unwrap();
            assert_eq!(register("dup").unwrap_err(), HealthError::DuplicateProbe);
        });
    }

    #[test]
    fn test_not_found_on_record() {
        run(|| {
            assert_eq!(
                record_result("missing", true, None, 0).unwrap_err(),
                HealthError::NotFound
            );
        });
    }

    #[test]
    fn test_deregister() {
        run(|| {
            register("temp").unwrap();
            deregister("temp").unwrap();
            assert_eq!(
                record_result("temp", true, None, 0).unwrap_err(),
                HealthError::NotFound
            );
        });
    }

    #[test]
    fn test_deregister_not_found() {
        run(|| {
            assert_eq!(deregister("ghost").unwrap_err(), HealthError::NotFound);
        });
    }

    #[test]
    fn test_invalid_name() {
        run(|| {
            assert_eq!(register("").unwrap_err(), HealthError::InvalidName);
            assert_eq!(register("  ").unwrap_err(), HealthError::InvalidName);
        });
    }

    #[test]
    fn test_all_probes() {
        run(|| {
            register("p1").unwrap();
            register("p2").unwrap();
            record_result("p1", true, Some("ok".to_string()), 50).unwrap();
            record_result("p2", false, None, 60).unwrap();
            let probes = all_probes().unwrap();
            assert_eq!(probes.len(), 2);
        });
    }

    #[test]
    fn test_empty_status_is_healthy() {
        run(|| {
            assert_eq!(status().unwrap(), OverallStatus::Healthy);
        });
    }

    #[test]
    fn test_message_recorded() {
        run(|| {
            register("msg-probe").unwrap();
            record_result("msg-probe", false, Some("disk full".to_string()), 999).unwrap();
            let probes = all_probes().unwrap();
            let p = probes.iter().find(|p| p.name == "msg-probe").unwrap();
            assert_eq!(p.message.as_deref(), Some("disk full"));
        });
    }
}
