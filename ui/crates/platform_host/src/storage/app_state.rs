//! App-state persistence contracts, envelope types, and helpers.

use std::{cell::RefCell, collections::HashMap, future::Future, pin::Pin, rc::Rc};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

/// Version for [`AppStateEnvelope`] metadata serialization.
pub const APP_STATE_ENVELOPE_VERSION: u32 = 1;
/// Namespace used by the desktop runtime durable snapshot.
pub const DESKTOP_STATE_NAMESPACE: &str = "system.desktop";
/// Namespace used by the calculator app state.
pub const CALCULATOR_STATE_NAMESPACE: &str = "app.calculator";
/// Namespace used by the notepad app state.
pub const NOTEPAD_STATE_NAMESPACE: &str = "app.notepad";
/// Namespace used by the explorer app state.
pub const EXPLORER_STATE_NAMESPACE: &str = "app.explorer";
/// Namespace used by the terminal app state.
pub const TERMINAL_STATE_NAMESPACE: &str = "app.terminal";
/// Namespace used by the paint placeholder app state.
pub const PAINT_STATE_NAMESPACE: &str = "app.paint";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Versioned envelope for persisted app state payloads.
pub struct AppStateEnvelope {
    /// Envelope schema version.
    pub envelope_version: u32,
    /// Namespace identifying the owning app/domain.
    pub namespace: String,
    /// App-defined schema version for the payload.
    pub schema_version: u32,
    /// Last update time in unix milliseconds.
    pub updated_at_unix_ms: u64,
    /// Serialized app payload.
    pub payload: Value,
}

impl AppStateEnvelope {
    /// Creates a new envelope and stamps it with a monotonic timestamp.
    pub fn new(namespace: impl Into<String>, schema_version: u32, payload: Value) -> Self {
        Self {
            envelope_version: APP_STATE_ENVELOPE_VERSION,
            namespace: namespace.into(),
            schema_version,
            updated_at_unix_ms: crate::time::next_monotonic_timestamp_ms(),
            payload,
        }
    }
}

/// Object-safe boxed future used by [`AppStateStore`] async methods.
pub type AppStateStoreFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Storage service for loading and saving app-state envelopes by namespace.
pub trait AppStateStore {
    /// Loads a persisted app-state envelope by namespace.
    fn load_app_state_envelope<'a>(
        &'a self,
        namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<Option<AppStateEnvelope>, String>>;

    /// Saves a full app-state envelope.
    fn save_app_state_envelope<'a>(
        &'a self,
        envelope: &'a AppStateEnvelope,
    ) -> AppStateStoreFuture<'a, Result<(), String>>;

    /// Deletes persisted app state for a namespace.
    fn delete_app_state<'a>(
        &'a self,
        namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<(), String>>;

    /// Lists namespaces currently present in the app-state store.
    fn list_app_state_namespaces<'a>(
        &'a self,
    ) -> AppStateStoreFuture<'a, Result<Vec<String>, String>>;
}

#[derive(Debug, Clone, Copy, Default)]
/// No-op app-state store for unsupported targets and baseline tests.
pub struct NoopAppStateStore;

impl AppStateStore for NoopAppStateStore {
    fn load_app_state_envelope<'a>(
        &'a self,
        _namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<Option<AppStateEnvelope>, String>> {
        Box::pin(async { Ok(None) })
    }

    fn save_app_state_envelope<'a>(
        &'a self,
        _envelope: &'a AppStateEnvelope,
    ) -> AppStateStoreFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }

    fn delete_app_state<'a>(
        &'a self,
        _namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }

    fn list_app_state_namespaces<'a>(
        &'a self,
    ) -> AppStateStoreFuture<'a, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}

#[derive(Debug, Clone)]
/// In-memory app-state store keyed by namespace.
pub struct MemoryAppStateStore {
    inner: Rc<RefCell<HashMap<String, AppStateEnvelope>>>,
}

impl Default for MemoryAppStateStore {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl AppStateStore for MemoryAppStateStore {
    fn load_app_state_envelope<'a>(
        &'a self,
        namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<Option<AppStateEnvelope>, String>> {
        Box::pin(async move { Ok(self.inner.borrow().get(namespace).cloned()) })
    }

    fn save_app_state_envelope<'a>(
        &'a self,
        envelope: &'a AppStateEnvelope,
    ) -> AppStateStoreFuture<'a, Result<(), String>> {
        Box::pin(async move {
            self.inner
                .borrow_mut()
                .insert(envelope.namespace.clone(), envelope.clone());
            Ok(())
        })
    }

    fn delete_app_state<'a>(
        &'a self,
        namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<(), String>> {
        Box::pin(async move {
            self.inner.borrow_mut().remove(namespace);
            Ok(())
        })
    }

    fn list_app_state_namespaces<'a>(
        &'a self,
    ) -> AppStateStoreFuture<'a, Result<Vec<String>, String>> {
        Box::pin(async move {
            let mut keys = self.inner.borrow().keys().cloned().collect::<Vec<_>>();
            keys.sort();
            Ok(keys)
        })
    }
}

/// Builds a versioned [`AppStateEnvelope`] from a serializable payload.
///
/// # Errors
///
/// Returns an error when `payload` cannot be converted to JSON.
pub fn build_app_state_envelope<T: Serialize>(
    namespace: &str,
    schema_version: u32,
    payload: &T,
) -> Result<AppStateEnvelope, String> {
    let payload = serde_json::to_value(payload).map_err(|e| e.to_string())?;
    Ok(AppStateEnvelope::new(
        namespace.to_string(),
        schema_version,
        payload,
    ))
}

/// Deserializes an envelope payload into a target type.
///
/// # Errors
///
/// Returns an error when deserialization fails.
pub fn migrate_envelope_payload<T: DeserializeOwned>(
    envelope: &AppStateEnvelope,
) -> Result<T, String> {
    serde_json::from_value(envelope.payload.clone()).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Schema compatibility policy used by [`load_app_state_typed_with`].
pub enum AppStateSchemaPolicy {
    /// Accept only an exact schema version.
    Exact(u32),
    /// Accept any schema version up to and including this maximum.
    UpTo(u32),
    /// Accept any schema version.
    Any,
}

impl AppStateSchemaPolicy {
    const fn allows(self, schema_version: u32) -> bool {
        match self {
            Self::Exact(expected) => schema_version == expected,
            Self::UpTo(max_supported) => schema_version <= max_supported,
            Self::Any => true,
        }
    }
}

/// Serializes and saves an app-state payload under `namespace`.
///
/// # Errors
///
/// Returns an error when payload serialization or storage fails.
pub async fn save_app_state_with<S: AppStateStore + ?Sized, T: Serialize>(
    store: &S,
    namespace: &str,
    schema_version: u32,
    payload: &T,
) -> Result<(), String> {
    let envelope = build_app_state_envelope(namespace, schema_version, payload)?;
    store.save_app_state_envelope(&envelope).await
}

/// Loads and deserializes typed app-state data through a specific store implementation.
///
/// Returns `Ok(None)` when:
/// - the namespace is not present
/// - envelope metadata version is incompatible
/// - the persisted schema version does not satisfy `schema_policy`
///
/// # Errors
///
/// Returns an error when the underlying storage load fails or payload deserialization fails.
pub async fn load_app_state_typed_with<S: AppStateStore + ?Sized, T: DeserializeOwned>(
    store: &S,
    namespace: &str,
    schema_policy: AppStateSchemaPolicy,
) -> Result<Option<T>, String> {
    let Some(envelope) = store.load_app_state_envelope(namespace).await? else {
        return Ok(None);
    };
    decode_typed_app_state_envelope(&envelope, schema_policy)
}

/// Loads typed app-state data while applying explicit legacy-schema migration hooks.
///
/// This is the preferred API for app/runtime hydration. It enforces envelope compatibility and
/// requires callers to handle legacy schemas intentionally instead of relying on broad
/// schema-policy acceptance.
///
/// # Errors
///
/// Returns an error when storage access fails, current-schema deserialization fails, or a caller
/// migration hook returns an error.
pub async fn load_app_state_with_migration<S, T, F>(
    store: &S,
    namespace: &str,
    current_schema_version: u32,
    migrate_legacy: F,
) -> Result<Option<T>, String>
where
    S: AppStateStore + ?Sized,
    T: DeserializeOwned,
    F: Fn(u32, &AppStateEnvelope) -> Result<Option<T>, String>,
{
    let Some(envelope) = store.load_app_state_envelope(namespace).await? else {
        return Ok(None);
    };
    decode_typed_app_state_with_migration(&envelope, current_schema_version, migrate_legacy)
}

fn decode_typed_app_state_envelope<T: DeserializeOwned>(
    envelope: &AppStateEnvelope,
    schema_policy: AppStateSchemaPolicy,
) -> Result<Option<T>, String> {
    if envelope.envelope_version != APP_STATE_ENVELOPE_VERSION {
        return Ok(None);
    }
    if !schema_policy.allows(envelope.schema_version) {
        return Ok(None);
    }
    migrate_envelope_payload(envelope).map(Some)
}

fn decode_typed_app_state_with_migration<T, F>(
    envelope: &AppStateEnvelope,
    current_schema_version: u32,
    migrate_legacy: F,
) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
    F: Fn(u32, &AppStateEnvelope) -> Result<Option<T>, String>,
{
    if envelope.envelope_version != APP_STATE_ENVELOPE_VERSION {
        return Ok(None);
    }

    if envelope.schema_version == current_schema_version {
        return migrate_envelope_payload(envelope).map(Some);
    }
    if envelope.schema_version > current_schema_version {
        return Ok(None);
    }
    migrate_legacy(envelope.schema_version, envelope)
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use serde::Deserialize;
    use serde::Serialize;
    use serde_json::json;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestPayload {
        count: u32,
        label: String,
    }

    #[test]
    fn app_state_envelope_serialization_shape_is_compatible() {
        let envelope = AppStateEnvelope {
            envelope_version: APP_STATE_ENVELOPE_VERSION,
            namespace: "app.example".to_string(),
            schema_version: 7,
            updated_at_unix_ms: 1234,
            payload: json!({"ok": true}),
        };

        let value = serde_json::to_value(&envelope).expect("serialize envelope");
        let object = value.as_object().expect("object");
        assert_eq!(object.get("envelope_version"), Some(&json!(1)));
        assert_eq!(object.get("namespace"), Some(&json!("app.example")));
        assert_eq!(object.get("schema_version"), Some(&json!(7)));
        assert_eq!(object.get("updated_at_unix_ms"), Some(&json!(1234)));
        assert_eq!(object.get("payload"), Some(&json!({"ok": true})));
        assert!(!object.contains_key("updatedAtUnixMs"));
    }

    #[test]
    fn app_state_envelope_new_uses_monotonic_timestamp() {
        let first = AppStateEnvelope::new("app.example", 1, json!({"n": 1}));
        let second = AppStateEnvelope::new("app.example", 1, json!({"n": 2}));
        assert!(second.updated_at_unix_ms > first.updated_at_unix_ms);
    }

    #[test]
    fn build_app_state_envelope_serializes_payload() {
        let envelope = build_app_state_envelope("app.example", 2, &json!({"answer": 42}))
            .expect("build envelope");
        assert_eq!(envelope.namespace, "app.example");
        assert_eq!(envelope.schema_version, 2);
        assert_eq!(envelope.payload, json!({"answer": 42}));
    }

    #[derive(Debug)]
    struct NonSerializable;

    impl Serialize for NonSerializable {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("boom"))
        }
    }

    #[test]
    fn build_app_state_envelope_returns_error_for_unserializable_payload() {
        let err = build_app_state_envelope("app.example", 1, &NonSerializable)
            .expect_err("expected serialization error");
        assert!(err.contains("boom"));
    }

    #[test]
    fn migrate_envelope_payload_round_trips() {
        let envelope = AppStateEnvelope {
            envelope_version: APP_STATE_ENVELOPE_VERSION,
            namespace: "app.example".to_string(),
            schema_version: 1,
            updated_at_unix_ms: 1,
            payload: json!({"count": 3, "label": "ok"}),
        };

        let decoded: TestPayload = migrate_envelope_payload(&envelope).expect("decode payload");
        assert_eq!(
            decoded,
            TestPayload {
                count: 3,
                label: "ok".to_string(),
            }
        );
    }

    #[test]
    fn migrate_envelope_payload_errors_on_type_mismatch() {
        let envelope = AppStateEnvelope {
            envelope_version: APP_STATE_ENVELOPE_VERSION,
            namespace: "app.example".to_string(),
            schema_version: 1,
            updated_at_unix_ms: 1,
            payload: json!({"count": "bad", "label": 7}),
        };

        let err = migrate_envelope_payload::<TestPayload>(&envelope)
            .expect_err("expected decode failure");
        assert!(!err.is_empty());
    }

    #[test]
    fn memory_app_state_store_round_trip_overwrite_delete_and_list() {
        let store = MemoryAppStateStore::default();
        let store_obj: &dyn AppStateStore = &store;

        let one = AppStateEnvelope {
            envelope_version: APP_STATE_ENVELOPE_VERSION,
            namespace: "app.one".to_string(),
            schema_version: 1,
            updated_at_unix_ms: 10,
            payload: json!({"v": 1}),
        };
        let one_updated = AppStateEnvelope {
            payload: json!({"v": 2}),
            ..one.clone()
        };
        let two = AppStateEnvelope {
            namespace: "app.two".to_string(),
            ..one.clone()
        };

        block_on(store_obj.save_app_state_envelope(&one)).expect("save one");
        block_on(store_obj.save_app_state_envelope(&two)).expect("save two");
        block_on(store_obj.save_app_state_envelope(&one_updated)).expect("overwrite one");

        let loaded = block_on(store_obj.load_app_state_envelope("app.one"))
            .expect("load")
            .expect("present");
        assert_eq!(loaded.payload, json!({"v": 2}));

        let namespaces = block_on(store_obj.list_app_state_namespaces()).expect("list");
        assert_eq!(
            namespaces,
            vec!["app.one".to_string(), "app.two".to_string()]
        );

        block_on(store_obj.delete_app_state("app.two")).expect("delete");
        assert_eq!(
            block_on(store_obj.load_app_state_envelope("app.two")).expect("load"),
            None
        );
    }

    #[test]
    fn noop_app_state_store_is_empty_and_successful() {
        let store = NoopAppStateStore;
        let store_obj: &dyn AppStateStore = &store;
        let envelope = AppStateEnvelope {
            envelope_version: APP_STATE_ENVELOPE_VERSION,
            namespace: "noop".to_string(),
            schema_version: 1,
            updated_at_unix_ms: 1,
            payload: json!({}),
        };

        assert_eq!(
            block_on(store_obj.load_app_state_envelope("noop")).expect("load"),
            None
        );
        block_on(store_obj.save_app_state_envelope(&envelope)).expect("save");
        block_on(store_obj.delete_app_state("noop")).expect("delete");
        assert_eq!(
            block_on(store_obj.list_app_state_namespaces()).expect("list"),
            Vec::<String>::new()
        );
    }
}
