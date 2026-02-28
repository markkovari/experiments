#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "batch-component",
    path: "../../wit/wasmcloud-batch",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use batch_core::{
    close as core_close, discard as core_discard, due_batches as core_due_batches,
    enqueue as core_enqueue, flush as core_flush, is_due as core_is_due, open as core_open,
    pending_count as core_pending_count, record_results as core_record_results,
    BatchError as CoreError, BatchItem as CoreItem, ItemResult as CoreItemResult,
};

#[cfg(target_arch = "wasm32")]
use wasmcloud::batch::types::{BatchError, BatchItem, FlushSummary, ItemResult};

#[cfg(target_arch = "wasm32")]
fn to_wit_err(e: CoreError) -> BatchError {
    match e {
        CoreError::BatchClosed    => BatchError::BatchClosed,
        CoreError::EmptyBatch     => BatchError::EmptyBatch,
        CoreError::InvalidId      => BatchError::InvalidId,
        CoreError::NotFound       => BatchError::NotFound,
        CoreError::DuplicateBatch => BatchError::DuplicateBatch,
    }
}

#[cfg(target_arch = "wasm32")]
fn from_wit_item(i: BatchItem) -> CoreItem {
    CoreItem { id: i.id, payload: i.payload, enqueued_at_ms: i.enqueued_at_ms }
}

#[cfg(target_arch = "wasm32")]
fn to_wit_item(i: CoreItem) -> BatchItem {
    BatchItem { id: i.id, payload: i.payload, enqueued_at_ms: i.enqueued_at_ms }
}

#[cfg(target_arch = "wasm32")]
fn from_wit_result(r: ItemResult) -> CoreItemResult {
    CoreItemResult { id: r.id, ok: r.ok, detail: r.detail }
}

#[cfg(target_arch = "wasm32")]
struct BatchComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::batch::batch_api::Guest for BatchComponent {
    fn open(name: String, max_size: u32, max_age_ms: u64) -> Result<(), BatchError> {
        core_open(&name, max_size, max_age_ms).map_err(to_wit_err)
    }

    fn enqueue(name: String, item: BatchItem) -> Result<(), BatchError> {
        core_enqueue(&name, from_wit_item(item)).map_err(to_wit_err)
    }

    fn flush(name: String, now_ms: u64) -> Result<Vec<BatchItem>, BatchError> {
        core_flush(&name, now_ms)
            .map(|items| items.into_iter().map(to_wit_item).collect())
            .map_err(to_wit_err)
    }

    fn record_results(name: String, results: Vec<ItemResult>) -> Result<FlushSummary, BatchError> {
        core_record_results(&name, results.into_iter().map(from_wit_result).collect())
            .map(|s| FlushSummary {
                total: s.total,
                succeeded: s.succeeded,
                failed: s.failed,
                flushed_at_ms: s.flushed_at_ms,
            })
            .map_err(to_wit_err)
    }

    fn is_due(name: String, now_ms: u64) -> Result<bool, BatchError> {
        core_is_due(&name, now_ms).map_err(to_wit_err)
    }

    fn pending_count(name: String) -> Result<u32, BatchError> {
        core_pending_count(&name).map_err(to_wit_err)
    }

    fn due_batches(now_ms: u64) -> Result<Vec<String>, BatchError> {
        core_due_batches(now_ms).map_err(to_wit_err)
    }

    fn discard(name: String) -> Result<(), BatchError> {
        core_discard(&name).map_err(to_wit_err)
    }

    fn close(name: String) -> Result<(), BatchError> {
        core_close(&name).map_err(to_wit_err)
    }
}

#[cfg(target_arch = "wasm32")]
export!(BatchComponent);

// ── native re-exports ──────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub use batch_core::{
    close, discard, due_batches, enqueue, flush, is_due, open, pending_count, record_results,
    BatchError, BatchItem, FlushSummary, ItemResult,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str) -> BatchItem {
        BatchItem { id: id.to_string(), payload: vec![1, 2, 3], enqueued_at_ms: 0 }
    }

    #[test]
    fn roundtrip() {
        std::thread::spawn(|| {
            open("comp-b1", 2, 0).unwrap();
            enqueue("comp-b1", item("alpha")).unwrap();
            enqueue("comp-b1", item("beta")).unwrap();
            assert!(is_due("comp-b1", 0).unwrap());
            let items = flush("comp-b1", 1000).unwrap();
            assert_eq!(items.len(), 2);
            let s = record_results("comp-b1", vec![
                ItemResult { id: "alpha".into(), ok: true, detail: None },
                ItemResult { id: "beta".into(), ok: true, detail: None },
            ]).unwrap();
            assert_eq!(s.succeeded, 2);
        }).join().unwrap();
    }
}
