use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub fields: Vec<(String, String)>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CounterSnapshot {
    pub name: String,
    pub value: u64,
    pub labels: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GaugeSnapshot {
    pub name: String,
    pub value: i64,
    pub labels: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObsError {
    NotInitialized,
    InvalidName,
}

// ── In-memory logger ──────────────────────────────────────────────────────────

struct LoggerState {
    min_level: LogLevel,
    buffer: Vec<LogEntry>,
}

thread_local! {
    static LOGGER: RefCell<LoggerState> = const { RefCell::new(LoggerState {
        min_level: LogLevel::Info,
        buffer: Vec::new(),
    }) };
}

fn with_logger<R>(f: impl FnOnce(&mut LoggerState) -> R) -> R {
    LOGGER.with(|l| f(&mut l.borrow_mut()))
}

/// Set the minimum log level. Entries below this are silently dropped.
pub fn set_level(level: LogLevel) -> Result<(), ObsError> {
    with_logger(|l| { l.min_level = level; Ok(()) })
}

/// Emit a structured log entry. Dropped when entry.level < min_level.
pub fn log(entry: LogEntry) -> Result<(), ObsError> {
    with_logger(|l| {
        if entry.level >= l.min_level {
            l.buffer.push(entry);
        }
        Ok(())
    })
}

/// Convenience: emit a plain message at the given level with no extra fields.
pub fn log_msg(level: LogLevel, target: &str, message: &str) -> Result<(), ObsError> {
    log(LogEntry {
        level,
        target: target.to_string(),
        message: message.to_string(),
        fields: vec![],
        timestamp_ms: 0,
    })
}

/// Drain and return all buffered log entries, clearing the buffer.
pub fn drain() -> Result<Vec<LogEntry>, ObsError> {
    with_logger(|l| Ok(std::mem::take(&mut l.buffer)))
}

// ── In-process metrics ────────────────────────────────────────────────────────

/// Key for a named metric with labels: `(name, sorted_label_pairs)`.
type MetricKey = (String, Vec<(String, String)>);

struct MetricsState {
    counters: HashMap<MetricKey, u64>,
    gauges: HashMap<MetricKey, i64>,
}

thread_local! {
    static METRICS: RefCell<MetricsState> = RefCell::new(MetricsState {
        counters: HashMap::new(),
        gauges: HashMap::new(),
    });
}

fn with_metrics<R>(f: impl FnOnce(&mut MetricsState) -> R) -> R {
    METRICS.with(|m| f(&mut m.borrow_mut()))
}

fn normalise_labels(labels: Vec<(String, String)>) -> Result<Vec<(String, String)>, ObsError> {
    let mut l = labels;
    l.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(l)
}

fn validate_name(name: &str) -> Result<(), ObsError> {
    if name.trim().is_empty() {
        return Err(ObsError::InvalidName);
    }
    Ok(())
}

/// Increment a named counter by `delta`.
pub fn increment(name: &str, delta: u64, labels: Vec<(String, String)>) -> Result<(), ObsError> {
    validate_name(name)?;
    let key = (name.to_string(), normalise_labels(labels)?);
    with_metrics(|m| {
        *m.counters.entry(key).or_insert(0) += delta;
        Ok(())
    })
}

/// Set a gauge to an absolute value.
pub fn gauge_set(name: &str, value: i64, labels: Vec<(String, String)>) -> Result<(), ObsError> {
    validate_name(name)?;
    let key = (name.to_string(), normalise_labels(labels)?);
    with_metrics(|m| {
        m.gauges.insert(key, value);
        Ok(())
    })
}

/// Return all counter snapshots.
pub fn counters() -> Result<Vec<CounterSnapshot>, ObsError> {
    with_metrics(|m| {
        Ok(m.counters
            .iter()
            .map(|((name, labels), &value)| CounterSnapshot {
                name: name.clone(),
                value,
                labels: labels.clone(),
            })
            .collect())
    })
}

/// Return all gauge snapshots.
pub fn gauges() -> Result<Vec<GaugeSnapshot>, ObsError> {
    with_metrics(|m| {
        Ok(m.gauges
            .iter()
            .map(|((name, labels), &value)| GaugeSnapshot {
                name: name.clone(),
                value,
                labels: labels.clone(),
            })
            .collect())
    })
}

/// Reset all counters and gauges to zero (clears maps).
pub fn reset() -> Result<(), ObsError> {
    with_metrics(|m| {
        m.counters.clear();
        m.gauges.clear();
        Ok(())
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Each test uses thread::spawn for isolation — thread_local state is
    // independent per OS thread, so tests never share logger/metrics state.

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_log_emit_and_drain() {
        run(|| {
            set_level(LogLevel::Debug).unwrap();
            log_msg(LogLevel::Info, "app.core", "hello").unwrap();
            log_msg(LogLevel::Debug, "app.core", "dbg").unwrap();
            let entries = drain().unwrap();
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].message, "hello");
            assert_eq!(entries[1].message, "dbg");
            // Drain again — buffer should be empty
            assert!(drain().unwrap().is_empty());
        });
    }

    #[test]
    fn test_log_level_filter() {
        run(|| {
            set_level(LogLevel::Warn).unwrap();
            log_msg(LogLevel::Info, "app", "should be dropped").unwrap();
            log_msg(LogLevel::Debug, "app", "also dropped").unwrap();
            log_msg(LogLevel::Warn, "app", "kept").unwrap();
            log_msg(LogLevel::Error, "app", "also kept").unwrap();
            let entries = drain().unwrap();
            assert_eq!(entries.len(), 2);
            assert!(entries.iter().all(|e| e.level >= LogLevel::Warn));
        });
    }

    #[test]
    fn test_log_structured_fields() {
        run(|| {
            set_level(LogLevel::Trace).unwrap();
            log(LogEntry {
                level: LogLevel::Info,
                target: "auth".to_string(),
                message: "request".to_string(),
                fields: vec![
                    ("user_id".to_string(), "42".to_string()),
                    ("method".to_string(), "GET".to_string()),
                ],
                timestamp_ms: 1000,
            })
            .unwrap();
            let entries = drain().unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].fields.len(), 2);
            assert_eq!(entries[0].timestamp_ms, 1000);
        });
    }

    #[test]
    fn test_counter_increment() {
        run(|| {
            increment("requests.total", 1, vec![]).unwrap();
            increment("requests.total", 1, vec![]).unwrap();
            increment("requests.total", 3, vec![]).unwrap();
            let snaps = counters().unwrap();
            let snap = snaps.iter().find(|s| s.name == "requests.total").unwrap();
            assert_eq!(snap.value, 5);
            reset().unwrap();
        });
    }

    #[test]
    fn test_counter_with_labels() {
        run(|| {
            let labels_get = vec![("method".to_string(), "GET".to_string())];
            let labels_post = vec![("method".to_string(), "POST".to_string())];
            increment("http.requests", 2, labels_get.clone()).unwrap();
            increment("http.requests", 1, labels_post.clone()).unwrap();
            let snaps = counters().unwrap();
            let get_snap =
                snaps.iter().find(|s| s.name == "http.requests" && s.labels == labels_get).unwrap();
            let post_snap =
                snaps.iter().find(|s| s.name == "http.requests" && s.labels == labels_post).unwrap();
            assert_eq!(get_snap.value, 2);
            assert_eq!(post_snap.value, 1);
            reset().unwrap();
        });
    }

    #[test]
    fn test_gauge_set() {
        run(|| {
            gauge_set("active.connections", 10, vec![]).unwrap();
            gauge_set("active.connections", 7, vec![]).unwrap(); // overwrite
            let snaps = gauges().unwrap();
            let snap = snaps.iter().find(|s| s.name == "active.connections").unwrap();
            assert_eq!(snap.value, 7);
            reset().unwrap();
        });
    }

    #[test]
    fn test_gauge_negative() {
        run(|| {
            gauge_set("queue.lag", -42, vec![]).unwrap();
            let snaps = gauges().unwrap();
            let snap = snaps.iter().find(|s| s.name == "queue.lag").unwrap();
            assert_eq!(snap.value, -42);
            reset().unwrap();
        });
    }

    #[test]
    fn test_reset_clears_all() {
        run(|| {
            increment("x", 5, vec![]).unwrap();
            gauge_set("y", 3, vec![]).unwrap();
            reset().unwrap();
            assert!(counters().unwrap().is_empty());
            assert!(gauges().unwrap().is_empty());
        });
    }

    #[test]
    fn test_invalid_metric_name() {
        run(|| {
            assert_eq!(increment("", 1, vec![]).unwrap_err(), ObsError::InvalidName);
            assert_eq!(increment("  ", 1, vec![]).unwrap_err(), ObsError::InvalidName);
            assert_eq!(gauge_set("", 1, vec![]).unwrap_err(), ObsError::InvalidName);
        });
    }

    #[test]
    fn test_labels_sorted_for_dedup() {
        run(|| {
            // Labels provided in different order must map to the same counter.
            increment("evt", 1, vec![
                ("b".to_string(), "2".to_string()),
                ("a".to_string(), "1".to_string()),
            ]).unwrap();
            increment("evt", 1, vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
            ]).unwrap();
            let snaps = counters().unwrap();
            let matching: Vec<_> = snaps.iter().filter(|s| s.name == "evt").collect();
            assert_eq!(matching.len(), 1, "different label orderings should deduplicate");
            assert_eq!(matching[0].value, 2);
            reset().unwrap();
        });
    }
}
