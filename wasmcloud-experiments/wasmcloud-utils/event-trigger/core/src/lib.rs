use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum EventError {
    InvalidName,
    NotFound,
    AlreadySubscribed,
    StorageError,
}

// ── Validation ────────────────────────────────────────────────────────────────

fn validate_name(name: &str) -> Result<String, EventError> {
    let n = name.trim().to_string();
    if n.is_empty() {
        return Err(EventError::InvalidName);
    }
    Ok(n)
}

// ── In-memory store ───────────────────────────────────────────────────────────

thread_local! {
    static STORE: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new());
}

fn with_store<R>(f: impl FnOnce(&mut HashMap<String, Vec<String>>) -> R) -> R {
    STORE.with(|s| f(&mut s.borrow_mut()))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Subscribe a function name to an event.
pub fn subscribe(event_name: &str, fn_name: &str) -> Result<(), EventError> {
    let event = validate_name(event_name)?;
    let func = validate_name(fn_name)?;
    with_store(|m| {
        let subs = m.entry(event).or_default();
        if subs.contains(&func) {
            return Err(EventError::AlreadySubscribed);
        }
        subs.push(func);
        Ok(())
    })
}

/// Unsubscribe a function name from an event.
pub fn unsubscribe(event_name: &str, fn_name: &str) -> Result<(), EventError> {
    let event = validate_name(event_name)?;
    let func = validate_name(fn_name)?;
    with_store(|m| {
        let subs = m.get_mut(&event).ok_or(EventError::NotFound)?;
        let pos = subs.iter().position(|s| s == &func).ok_or(EventError::NotFound)?;
        subs.remove(pos);
        Ok(())
    })
}

/// Emit an event. Returns the list of subscribed function names.
pub fn emit(event_name: &str, _payload: &[u8]) -> Result<Vec<String>, EventError> {
    let event = validate_name(event_name)?;
    with_store(|m| Ok(m.get(&event).cloned().unwrap_or_default()))
}

/// List all function names subscribed to an event.
pub fn subscribers(event_name: &str) -> Result<Vec<String>, EventError> {
    let event = validate_name(event_name)?;
    with_store(|m| Ok(m.get(&event).cloned().unwrap_or_default()))
}

/// List all registered event names.
pub fn all_events() -> Result<Vec<String>, EventError> {
    with_store(|m| Ok(m.keys().cloned().collect()))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_subscribe_and_emit() {
        run(|| {
            subscribe("order.created", "send-email").unwrap();
            subscribe("order.created", "update-inventory").unwrap();
            let fns = emit("order.created", b"{}").unwrap();
            assert_eq!(fns.len(), 2);
            assert!(fns.contains(&"send-email".to_string()));
            assert!(fns.contains(&"update-inventory".to_string()));
        });
    }

    #[test]
    fn test_emit_unknown_event_returns_empty() {
        run(|| {
            let fns = emit("unknown.event", b"{}").unwrap();
            assert!(fns.is_empty());
        });
    }

    #[test]
    fn test_duplicate_subscribe_errors() {
        run(|| {
            subscribe("dup.event", "fn-a").unwrap();
            assert_eq!(subscribe("dup.event", "fn-a").unwrap_err(), EventError::AlreadySubscribed);
        });
    }

    #[test]
    fn test_unsubscribe() {
        run(|| {
            subscribe("unsub.event", "fn-x").unwrap();
            unsubscribe("unsub.event", "fn-x").unwrap();
            let fns = subscribers("unsub.event").unwrap();
            assert!(fns.is_empty());
        });
    }

    #[test]
    fn test_unsubscribe_not_found() {
        run(|| {
            assert_eq!(unsubscribe("no.event", "fn-y").unwrap_err(), EventError::NotFound);
        });
    }

    #[test]
    fn test_subscribers() {
        run(|| {
            subscribe("subs.event", "fn-1").unwrap();
            subscribe("subs.event", "fn-2").unwrap();
            let subs = subscribers("subs.event").unwrap();
            assert_eq!(subs.len(), 2);
        });
    }

    #[test]
    fn test_all_events() {
        run(|| {
            subscribe("evt.a", "fn-1").unwrap();
            subscribe("evt.b", "fn-2").unwrap();
            let evts = all_events().unwrap();
            assert!(evts.contains(&"evt.a".to_string()));
            assert!(evts.contains(&"evt.b".to_string()));
        });
    }

    #[test]
    fn test_invalid_name() {
        run(|| {
            assert_eq!(subscribe("", "fn-1").unwrap_err(), EventError::InvalidName);
            assert_eq!(subscribe("evt", "").unwrap_err(), EventError::InvalidName);
        });
    }
}
