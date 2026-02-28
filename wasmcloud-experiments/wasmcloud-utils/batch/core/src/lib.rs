use std::cell::RefCell;
use std::collections::HashMap;

// ── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BatchItem {
    pub id: String,
    pub payload: Vec<u8>,
    pub enqueued_at_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ItemResult {
    pub id: String,
    pub ok: bool,
    pub detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FlushSummary {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub flushed_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatchError {
    BatchClosed,
    EmptyBatch,
    InvalidId,
    NotFound,
    DuplicateBatch,
}

#[derive(Debug, Clone, PartialEq)]
enum WindowState {
    Open,
    Flushed,
}

#[derive(Debug, Clone)]
struct BatchWindow {
    max_size: u32,
    max_age_ms: u64,
    /// unix-ms when the first item was enqueued (None = still empty)
    first_enqueue_ms: Option<u64>,
    items: Vec<BatchItem>,
    state: WindowState,
    /// results recorded after a flush (for flush-summary)
    results: Vec<ItemResult>,
    flush_ms: u64,
}

// ── Thread-local store ─────────────────────────────────────────────────────

thread_local! {
    static BATCHES: RefCell<HashMap<String, BatchWindow>> = RefCell::new(HashMap::new());
}

fn with_batches<R>(f: impl FnOnce(&mut HashMap<String, BatchWindow>) -> R) -> R {
    BATCHES.with(|b| f(&mut b.borrow_mut()))
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn validate_id(id: &str) -> Result<(), BatchError> {
    if id.trim().is_empty() {
        Err(BatchError::InvalidId)
    } else {
        Ok(())
    }
}

impl BatchWindow {
    fn new(max_size: u32, max_age_ms: u64) -> Self {
        BatchWindow {
            max_size,
            max_age_ms,
            first_enqueue_ms: None,
            items: Vec::new(),
            state: WindowState::Open,
            results: Vec::new(),
            flush_ms: 0,
        }
    }

    fn is_due(&self, now_ms: u64) -> bool {
        if self.state != WindowState::Open {
            return false;
        }
        // size trigger
        if self.max_size > 0 && self.items.len() as u32 >= self.max_size {
            return true;
        }
        // age trigger
        if self.max_age_ms > 0 {
            if let Some(first) = self.first_enqueue_ms {
                if now_ms.saturating_sub(first) >= self.max_age_ms {
                    return true;
                }
            }
        }
        false
    }
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Create a named batch window.
pub fn open(name: &str, max_size: u32, max_age_ms: u64) -> Result<(), BatchError> {
    let name = name.trim().to_string();
    validate_id(&name)?;
    with_batches(|m| {
        if m.contains_key(&name) {
            return Err(BatchError::DuplicateBatch);
        }
        m.insert(name, BatchWindow::new(max_size, max_age_ms));
        Ok(())
    })
}

/// Enqueue an item into the named batch.
pub fn enqueue(name: &str, item: BatchItem) -> Result<(), BatchError> {
    validate_id(&item.id)?;
    with_batches(|m| {
        let w = m.get_mut(name).ok_or(BatchError::NotFound)?;
        if w.state != WindowState::Open {
            return Err(BatchError::BatchClosed);
        }
        if w.first_enqueue_ms.is_none() {
            w.first_enqueue_ms = Some(item.enqueued_at_ms);
        }
        w.items.push(item);
        Ok(())
    })
}

/// Flush — returns all pending items and marks the window closed.
pub fn flush(name: &str, now_ms: u64) -> Result<Vec<BatchItem>, BatchError> {
    with_batches(|m| {
        let w = m.get_mut(name).ok_or(BatchError::NotFound)?;
        if w.state != WindowState::Open {
            return Err(BatchError::BatchClosed);
        }
        if w.items.is_empty() {
            return Err(BatchError::EmptyBatch);
        }
        w.state = WindowState::Flushed;
        w.flush_ms = now_ms;
        Ok(std::mem::take(&mut w.items))
    })
}

/// Record outcomes for items that were returned by flush.
pub fn record_results(name: &str, results: Vec<ItemResult>) -> Result<FlushSummary, BatchError> {
    with_batches(|m| {
        let w = m.get_mut(name).ok_or(BatchError::NotFound)?;
        let total = results.len() as u32;
        let succeeded = results.iter().filter(|r| r.ok).count() as u32;
        let failed = total - succeeded;
        w.results = results;
        Ok(FlushSummary {
            total,
            succeeded,
            failed,
            flushed_at_ms: w.flush_ms,
        })
    })
}

/// Is this batch due for auto-flush at now_ms?
pub fn is_due(name: &str, now_ms: u64) -> Result<bool, BatchError> {
    with_batches(|m| {
        let w = m.get(name).ok_or(BatchError::NotFound)?;
        Ok(w.is_due(now_ms))
    })
}

/// How many items are currently pending?
pub fn pending_count(name: &str) -> Result<u32, BatchError> {
    with_batches(|m| {
        let w = m.get(name).ok_or(BatchError::NotFound)?;
        Ok(w.items.len() as u32)
    })
}

/// All batch names that are due at now_ms.
pub fn due_batches(now_ms: u64) -> Result<Vec<String>, BatchError> {
    with_batches(|m| {
        Ok(m.iter()
            .filter(|(_, w)| w.is_due(now_ms))
            .map(|(name, _)| name.clone())
            .collect())
    })
}

/// Discard all pending items, close the window.
pub fn discard(name: &str) -> Result<(), BatchError> {
    with_batches(|m| {
        let w = m.get_mut(name).ok_or(BatchError::NotFound)?;
        w.items.clear();
        w.state = WindowState::Flushed;
        Ok(())
    })
}

/// Remove the batch entirely.
pub fn close(name: &str) -> Result<(), BatchError> {
    with_batches(|m| {
        m.remove(name).ok_or(BatchError::NotFound).map(|_| ())
    })
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::spawn(f).join().unwrap();
    }

    fn item(id: &str, at: u64) -> BatchItem {
        BatchItem { id: id.to_string(), payload: id.as_bytes().to_vec(), enqueued_at_ms: at }
    }

    #[test]
    fn test_open_and_enqueue() {
        run(|| {
            open("q1", 10, 0).unwrap();
            enqueue("q1", item("a", 1000)).unwrap();
            enqueue("q1", item("b", 1001)).unwrap();
            assert_eq!(pending_count("q1").unwrap(), 2);
        });
    }

    #[test]
    fn test_flush_returns_items_in_order() {
        run(|| {
            open("q2", 0, 0).unwrap();
            enqueue("q2", item("x", 0)).unwrap();
            enqueue("q2", item("y", 0)).unwrap();
            let items = flush("q2", 5000).unwrap();
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].id, "x");
            assert_eq!(items[1].id, "y");
        });
    }

    #[test]
    fn test_flush_closes_window() {
        run(|| {
            open("q3", 0, 0).unwrap();
            enqueue("q3", item("a", 0)).unwrap();
            flush("q3", 0).unwrap();
            // second flush should fail
            assert_eq!(flush("q3", 0).unwrap_err(), BatchError::BatchClosed);
        });
    }

    #[test]
    fn test_flush_empty_batch_fails() {
        run(|| {
            open("q4", 0, 0).unwrap();
            assert_eq!(flush("q4", 0).unwrap_err(), BatchError::EmptyBatch);
        });
    }

    #[test]
    fn test_size_trigger() {
        run(|| {
            open("q5", 3, 0).unwrap();
            assert!(!is_due("q5", 0).unwrap());
            enqueue("q5", item("a", 0)).unwrap();
            enqueue("q5", item("b", 0)).unwrap();
            assert!(!is_due("q5", 0).unwrap());
            enqueue("q5", item("c", 0)).unwrap();
            assert!(is_due("q5", 0).unwrap()); // 3 >= max_size 3
        });
    }

    #[test]
    fn test_age_trigger() {
        run(|| {
            open("q6", 0, 500).unwrap(); // 500 ms max age
            enqueue("q6", item("a", 1000)).unwrap();
            assert!(!is_due("q6", 1000).unwrap()); // 0 ms elapsed
            assert!(!is_due("q6", 1499).unwrap()); // 499 ms elapsed
            assert!(is_due("q6", 1500).unwrap());  // 500 ms elapsed
        });
    }

    #[test]
    fn test_due_batches() {
        run(|| {
            open("qa", 2, 0).unwrap();
            open("qb", 2, 0).unwrap();
            enqueue("qa", item("1", 0)).unwrap();
            enqueue("qa", item("2", 0)).unwrap(); // qa is due
            enqueue("qb", item("1", 0)).unwrap(); // qb not due
            let due = due_batches(0).unwrap();
            assert!(due.contains(&"qa".to_string()));
            assert!(!due.contains(&"qb".to_string()));
        });
    }

    #[test]
    fn test_record_results_summary() {
        run(|| {
            open("qr", 0, 0).unwrap();
            enqueue("qr", item("a", 0)).unwrap();
            enqueue("qr", item("b", 0)).unwrap();
            enqueue("qr", item("c", 0)).unwrap();
            flush("qr", 9000).unwrap();
            let summary = record_results("qr", vec![
                ItemResult { id: "a".into(), ok: true, detail: None },
                ItemResult { id: "b".into(), ok: false, detail: Some("timeout".into()) },
                ItemResult { id: "c".into(), ok: true, detail: None },
            ]).unwrap();
            assert_eq!(summary.total, 3);
            assert_eq!(summary.succeeded, 2);
            assert_eq!(summary.failed, 1);
            assert_eq!(summary.flushed_at_ms, 9000);
        });
    }

    #[test]
    fn test_discard() {
        run(|| {
            open("qd", 0, 0).unwrap();
            enqueue("qd", item("a", 0)).unwrap();
            discard("qd").unwrap();
            assert_eq!(pending_count("qd").unwrap(), 0);
            assert_eq!(flush("qd", 0).unwrap_err(), BatchError::BatchClosed);
        });
    }

    #[test]
    fn test_enqueue_after_flush_fails() {
        run(|| {
            open("qe", 0, 0).unwrap();
            enqueue("qe", item("a", 0)).unwrap();
            flush("qe", 0).unwrap();
            assert_eq!(
                enqueue("qe", item("b", 0)).unwrap_err(),
                BatchError::BatchClosed
            );
        });
    }

    #[test]
    fn test_duplicate_open() {
        run(|| {
            open("qdupe", 0, 0).unwrap();
            assert_eq!(open("qdupe", 0, 0).unwrap_err(), BatchError::DuplicateBatch);
        });
    }

    #[test]
    fn test_close_removes_batch() {
        run(|| {
            open("qclose", 0, 0).unwrap();
            close("qclose").unwrap();
            assert_eq!(pending_count("qclose").unwrap_err(), BatchError::NotFound);
        });
    }

    #[test]
    fn test_invalid_item_id() {
        run(|| {
            open("qinv", 0, 0).unwrap();
            assert_eq!(
                enqueue("qinv", BatchItem { id: "".into(), payload: vec![], enqueued_at_ms: 0 })
                    .unwrap_err(),
                BatchError::InvalidId
            );
        });
    }
}
