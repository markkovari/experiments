// WIT-based tracing component.
// Targets the `tracing-component` world defined in wit/wasmcloud-tracing/tracing.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "tracing-component",
    path: "../../wit/wasmcloud-tracing",
    generate_all,
});

#[cfg(target_arch = "wasm32")]
use tracing_core::{
    active_spans as core_active_spans, add_tag as core_add_tag, current_span_id,
    end_span as core_end_span, get_span as core_get_span, start_span as core_start_span,
    TracingError as CoreError,
};

#[allow(dead_code)]
fn now_ms() -> u64 {
    0
}

#[cfg(target_arch = "wasm32")]
fn core_err(e: CoreError) -> wasmcloud::tracing::types::TracingError {
    use wasmcloud::tracing::types::TracingError;
    match e {
        CoreError::InvalidSpanId => TracingError::InvalidSpanId,
        CoreError::NotFound => TracingError::NotFound,
        CoreError::InvalidName => TracingError::InvalidName,
    }
}

#[cfg(target_arch = "wasm32")]
fn wit_span(s: tracing_core::Span) -> wasmcloud::tracing::types::Span {
    wasmcloud::tracing::types::Span {
        id: s.id,
        parent_id: s.parent_id,
        name: s.name,
        started_ms: s.started_ms,
        ended_ms: s.ended_ms,
        tags: s.tags,
    }
}

#[cfg(target_arch = "wasm32")]
struct TracingComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::tracing::tracing_api::Guest for TracingComponent {
    fn start_span(
        name: String,
        parent_id: Option<String>,
    ) -> Result<String, wasmcloud::tracing::types::TracingError> {
        core_start_span(&name, parent_id, now_ms()).map_err(core_err)
    }

    fn end_span(span_id: String) -> Result<(), wasmcloud::tracing::types::TracingError> {
        core_end_span(&span_id, now_ms()).map_err(core_err)
    }

    fn current_span() -> Result<Option<String>, wasmcloud::tracing::types::TracingError> {
        current_span_id().map_err(core_err)
    }

    fn add_tag(
        span_id: String,
        key: String,
        value: String,
    ) -> Result<(), wasmcloud::tracing::types::TracingError> {
        core_add_tag(&span_id, &key, &value).map_err(core_err)
    }

    fn get_span(
        span_id: String,
    ) -> Result<wasmcloud::tracing::types::Span, wasmcloud::tracing::types::TracingError> {
        core_get_span(&span_id).map(wit_span).map_err(core_err)
    }

    fn active_spans(
    ) -> Result<Vec<wasmcloud::tracing::types::Span>, wasmcloud::tracing::types::TracingError> {
        core_active_spans().map(|v| v.into_iter().map(wit_span).collect()).map_err(core_err)
    }
}

#[cfg(target_arch = "wasm32")]
export!(TracingComponent);

// ── native helpers ────────────────────────────────────────────────────────────

pub use tracing_core::{
    active_spans, add_tag, current_span_id as current_span, end_span, get_span, start_span,
    TracingError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        std::thread::spawn(|| {
            let id = start_span("root", None, 0).unwrap();
            add_tag(&id, "service", "api").unwrap();
            let span = get_span(&id).unwrap();
            assert_eq!(span.name, "root");
            assert_eq!(span.tags[0], ("service".to_string(), "api".to_string()));
            end_span(&id, 100).unwrap();
            let ended = get_span(&id).unwrap();
            assert_eq!(ended.ended_ms, Some(100));
        })
        .join()
        .unwrap();
    }

    #[test]
    fn current_span_lifecycle() {
        std::thread::spawn(|| {
            assert!(current_span().unwrap().is_none());
            let id = start_span("lifecycle", None, 0).unwrap();
            assert_eq!(current_span().unwrap().as_deref(), Some(id.as_str()));
            end_span(&id, 1).unwrap();
            assert!(current_span().unwrap().is_none());
        })
        .join()
        .unwrap();
    }
}
