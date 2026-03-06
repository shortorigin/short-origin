//! Tauri command-backed preference store transport.
//!
//! This store uses the bridge interop layer, which routes preference calls to Tauri commands
//! when available in desktop webview contexts.

use platform_host::{PrefsStore, PrefsStoreFuture};

#[derive(Debug, Clone, Copy, Default)]
/// Desktop preference store backed by Tauri command transport.
pub struct TauriPrefsStore;

impl PrefsStore for TauriPrefsStore {
    fn load_pref<'a>(
        &'a self,
        key: &'a str,
    ) -> PrefsStoreFuture<'a, Result<Option<String>, String>> {
        Box::pin(async move { crate::bridge::load_pref(key).await })
    }

    fn save_pref<'a>(
        &'a self,
        key: &'a str,
        raw_json: &'a str,
    ) -> PrefsStoreFuture<'a, Result<(), String>> {
        Box::pin(async move { crate::bridge::save_pref(key, raw_json).await })
    }

    fn delete_pref<'a>(&'a self, key: &'a str) -> PrefsStoreFuture<'a, Result<(), String>> {
        Box::pin(async move { crate::bridge::delete_pref(key).await })
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_tauri_prefs_adapter_matches_bridge_fallback_behavior() {
        let store = TauriPrefsStore;
        let store_obj: &dyn PrefsStore = &store;

        assert_eq!(
            block_on(store_obj.load_pref("retrodesk.key")).expect("load"),
            None
        );
        block_on(store_obj.save_pref("retrodesk.key", "\"value\"")).expect("save");
        block_on(store_obj.delete_pref("retrodesk.key")).expect("delete");
    }
}
