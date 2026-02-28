use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Span {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub started_ms: u64,
    pub ended_ms: Option<u64>,
    pub tags: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TracingError {
    InvalidSpanId,
    NotFound,
    InvalidName,
}

// ── ID generation (deterministic counter, no OS entropy) ─────────────────────

static SPAN_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_span_id() -> String {
    let n = SPAN_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("span-{:016x}", n)
}

// ── Thread-local state ────────────────────────────────────────────────────────

struct TracingState {
    spans: HashMap<String, Span>,
    stack: Vec<String>,
}

thread_local! {
    static STATE: RefCell<TracingState> = RefCell::new(TracingState {
        spans: HashMap::new(),
        stack: Vec::new(),
    });
}

fn with_state<R>(f: impl FnOnce(&mut TracingState) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

// ── Public API ─────────────────────────────────────────────────────────────────

fn validate_name(name: &str) -> Result<(), TracingError> {
    if name.trim().is_empty() {
        return Err(TracingError::InvalidName);
    }
    Ok(())
}

/// Start a new span with the given name and optional parent ID.
/// Returns the new span's ID.
pub fn start_span(name: &str, parent_id: Option<String>, now_ms: u64) -> Result<String, TracingError> {
    validate_name(name)?;
    if let Some(ref pid) = parent_id {
        if pid.trim().is_empty() {
            return Err(TracingError::InvalidSpanId);
        }
    }
    let id = next_span_id();
    let span = Span {
        id: id.clone(),
        parent_id,
        name: name.to_string(),
        started_ms: now_ms,
        ended_ms: None,
        tags: Vec::new(),
    };
    with_state(|s| {
        s.spans.insert(id.clone(), span);
        s.stack.push(id.clone());
    });
    Ok(id)
}

/// End a span by ID, recording its end time.
pub fn end_span(span_id: &str, now_ms: u64) -> Result<(), TracingError> {
    if span_id.trim().is_empty() {
        return Err(TracingError::InvalidSpanId);
    }
    with_state(|s| {
        let span = s.spans.get_mut(span_id).ok_or(TracingError::NotFound)?;
        span.ended_ms = Some(now_ms);
        s.stack.retain(|id| id != span_id);
        Ok(())
    })
}

/// Return the ID of the current (top of stack) span, if any.
pub fn current_span_id() -> Result<Option<String>, TracingError> {
    Ok(with_state(|s| s.stack.last().cloned()))
}

/// Add a tag to a span.
pub fn add_tag(span_id: &str, key: &str, value: &str) -> Result<(), TracingError> {
    if span_id.trim().is_empty() {
        return Err(TracingError::InvalidSpanId);
    }
    with_state(|s| {
        let span = s.spans.get_mut(span_id).ok_or(TracingError::NotFound)?;
        span.tags.push((key.to_string(), value.to_string()));
        Ok(())
    })
}

/// Get a span by ID.
pub fn get_span(span_id: &str) -> Result<Span, TracingError> {
    if span_id.trim().is_empty() {
        return Err(TracingError::InvalidSpanId);
    }
    with_state(|s| s.spans.get(span_id).cloned().ok_or(TracingError::NotFound))
}

/// Return all spans that have not yet ended.
pub fn active_spans() -> Result<Vec<Span>, TracingError> {
    Ok(with_state(|s| {
        s.spans.values().filter(|sp| sp.ended_ms.is_none()).cloned().collect()
    }))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_start_span_returns_id() {
        run(|| {
            let id = start_span("request", None, 1000).unwrap();
            assert!(!id.is_empty());
        });
    }

    #[test]
    fn test_end_span_records_time() {
        run(|| {
            let id = start_span("op", None, 100).unwrap();
            end_span(&id, 200).unwrap();
            let span = get_span(&id).unwrap();
            assert_eq!(span.ended_ms, Some(200));
        });
    }

    #[test]
    fn test_current_span_tracks_stack() {
        run(|| {
            assert!(current_span_id().unwrap().is_none());
            let id = start_span("outer", None, 0).unwrap();
            assert_eq!(current_span_id().unwrap().as_deref(), Some(id.as_str()));
            let id2 = start_span("inner", Some(id.clone()), 0).unwrap();
            assert_eq!(current_span_id().unwrap().as_deref(), Some(id2.as_str()));
            end_span(&id2, 1).unwrap();
            assert_eq!(current_span_id().unwrap().as_deref(), Some(id.as_str()));
        });
    }

    #[test]
    fn test_add_tag() {
        run(|| {
            let id = start_span("tagged", None, 0).unwrap();
            add_tag(&id, "env", "prod").unwrap();
            let span = get_span(&id).unwrap();
            assert!(span.tags.iter().any(|(k, v)| k == "env" && v == "prod"));
        });
    }

    #[test]
    fn test_get_span_not_found() {
        run(|| {
            assert_eq!(get_span("nonexistent").unwrap_err(), TracingError::NotFound);
        });
    }

    #[test]
    fn test_invalid_name() {
        run(|| {
            assert_eq!(start_span("", None, 0).unwrap_err(), TracingError::InvalidName);
            assert_eq!(start_span("   ", None, 0).unwrap_err(), TracingError::InvalidName);
        });
    }

    #[test]
    fn test_invalid_span_id() {
        run(|| {
            assert_eq!(end_span("", 0).unwrap_err(), TracingError::InvalidSpanId);
            assert_eq!(add_tag("", "k", "v").unwrap_err(), TracingError::InvalidSpanId);
            assert_eq!(get_span("").unwrap_err(), TracingError::InvalidSpanId);
        });
    }

    #[test]
    fn test_active_spans_excludes_ended() {
        run(|| {
            let id1 = start_span("a1", None, 0).unwrap();
            let id2 = start_span("a2", None, 0).unwrap();
            end_span(&id1, 10).unwrap();
            let active = active_spans().unwrap();
            assert!(!active.iter().any(|s| s.id == id1));
            assert!(active.iter().any(|s| s.id == id2));
        });
    }

    #[test]
    fn test_parent_id_recorded() {
        run(|| {
            let parent = start_span("parent", None, 0).unwrap();
            let child = start_span("child", Some(parent.clone()), 0).unwrap();
            let span = get_span(&child).unwrap();
            assert_eq!(span.parent_id.as_deref(), Some(parent.as_str()));
        });
    }

    #[test]
    fn test_multiple_tags() {
        run(|| {
            let id = start_span("multi-tag", None, 0).unwrap();
            add_tag(&id, "a", "1").unwrap();
            add_tag(&id, "b", "2").unwrap();
            add_tag(&id, "c", "3").unwrap();
            let span = get_span(&id).unwrap();
            assert_eq!(span.tags.len(), 3);
        });
    }

    #[test]
    fn test_ids_are_unique() {
        run(|| {
            let id1 = start_span("s1", None, 0).unwrap();
            let id2 = start_span("s2", None, 0).unwrap();
            assert_ne!(id1, id2);
        });
    }
}
