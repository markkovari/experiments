// WIT-based event-trigger component.
// Targets the `event-trigger-component` world defined in wit/wasmcloud-event-trigger/event-trigger.wit.

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    world: "event-trigger-component",
    path: "../../wit/wasmcloud-event-trigger",
    generate_all,
});

use event_trigger_core::{
    all_events as core_all_events, emit as core_emit, subscribe as core_subscribe,
    subscribers as core_subscribers, unsubscribe as core_unsubscribe, EventError as CoreError,
};

// ---- type conversions (wasm32 only) -----------------------------------------

#[cfg(target_arch = "wasm32")]
fn core_error(e: CoreError) -> wasmcloud::event_trigger::types::EventError {
    use wasmcloud::event_trigger::types::EventError;
    match e {
        CoreError::InvalidName => EventError::InvalidName,
        CoreError::NotFound => EventError::NotFound,
        CoreError::AlreadySubscribed => EventError::AlreadySubscribed,
        CoreError::StorageError => EventError::StorageError,
    }
}

// ---- WIT guest implementation -----------------------------------------------

#[cfg(target_arch = "wasm32")]
struct EventTriggerComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::event_trigger::event_api::Guest for EventTriggerComponent {
    fn subscribe(
        event_name: String,
        fn_name: String,
    ) -> Result<(), wasmcloud::event_trigger::types::EventError> {
        core_subscribe(&event_name, &fn_name).map_err(core_error)
    }

    fn unsubscribe(
        event_name: String,
        fn_name: String,
    ) -> Result<(), wasmcloud::event_trigger::types::EventError> {
        core_unsubscribe(&event_name, &fn_name).map_err(core_error)
    }

    fn emit(
        event_name: String,
        payload: Vec<u8>,
    ) -> Result<Vec<String>, wasmcloud::event_trigger::types::EventError> {
        core_emit(&event_name, &payload).map_err(core_error)
    }

    fn subscribers(
        event_name: String,
    ) -> Result<Vec<String>, wasmcloud::event_trigger::types::EventError> {
        core_subscribers(&event_name).map_err(core_error)
    }

    fn all_events() -> Result<Vec<String>, wasmcloud::event_trigger::types::EventError> {
        core_all_events().map_err(core_error)
    }
}

#[cfg(target_arch = "wasm32")]
export!(EventTriggerComponent);

// ---- native helpers (cargo check / tests) -----------------------------------

pub fn event_subscribe(event_name: &str, fn_name: &str) -> Result<(), CoreError> {
    core_subscribe(event_name, fn_name)
}

pub fn event_emit(event_name: &str, payload: &[u8]) -> Result<Vec<String>, CoreError> {
    core_emit(event_name, payload)
}

pub fn event_subscribers(event_name: &str) -> Result<Vec<String>, CoreError> {
    core_subscribers(event_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        event_subscribe("comp.evt", "comp-fn").unwrap();
        let fns = event_emit("comp.evt", b"payload").unwrap();
        assert!(fns.contains(&"comp-fn".to_string()));
    }

    #[test]
    fn subscribers_roundtrip() {
        event_subscribe("sub.evt", "sub-fn-1").unwrap();
        event_subscribe("sub.evt", "sub-fn-2").unwrap();
        let subs = event_subscribers("sub.evt").unwrap();
        assert_eq!(subs.len(), 2);
    }
}
