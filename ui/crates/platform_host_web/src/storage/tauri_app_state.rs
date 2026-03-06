//! Tauri command-backed app-state store transport.
//!
//! This store uses the bridge interop layer, which routes app-state calls to Tauri
//! commands when available in desktop webview contexts.

use platform_host::{AppStateEnvelope, AppStateStore, AppStateStoreFuture};

#[derive(Debug, Clone, Copy, Default)]
/// Desktop app-state store backed by Tauri command transport.
pub struct TauriAppStateStore;

impl AppStateStore for TauriAppStateStore {
    fn load_app_state_envelope<'a>(
        &'a self,
        namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<Option<AppStateEnvelope>, String>> {
        Box::pin(async move { crate::bridge::load_app_state_envelope(namespace).await })
    }

    fn save_app_state_envelope<'a>(
        &'a self,
        envelope: &'a AppStateEnvelope,
    ) -> AppStateStoreFuture<'a, Result<(), String>> {
        Box::pin(async move { crate::bridge::save_app_state_envelope(envelope).await })
    }

    fn delete_app_state<'a>(
        &'a self,
        namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<(), String>> {
        Box::pin(async move { crate::bridge::delete_app_state(namespace).await })
    }

    fn list_app_state_namespaces<'a>(
        &'a self,
    ) -> AppStateStoreFuture<'a, Result<Vec<String>, String>> {
        Box::pin(async move { crate::bridge::list_app_state_namespaces().await })
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use serde_json::json;

    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_tauri_app_state_adapter_matches_bridge_fallback_behavior() {
        let store = TauriAppStateStore;
        let store_obj: &dyn AppStateStore = &store;

        let envelope = AppStateEnvelope {
            envelope_version: 1,
            namespace: "app.example".to_string(),
            schema_version: 1,
            updated_at_unix_ms: 1,
            payload: json!({"ok": true}),
        };

        assert_eq!(
            block_on(store_obj.load_app_state_envelope("app.example")).expect("load"),
            None
        );
        block_on(store_obj.save_app_state_envelope(&envelope)).expect("save");
        block_on(store_obj.delete_app_state("app.example")).expect("delete");
        assert_eq!(
            block_on(store_obj.list_app_state_namespaces()).expect("list"),
            Vec::<String>::new()
        );
    }
}
