//! Browser preference storage implementation backed by the shared async bridge.

use platform_host::{HostResult, PrefsStore, PrefsStoreFuture};

#[derive(Debug, Clone, Copy, Default)]
/// Browser preference store backed by browser persistence through the shared bridge.
pub struct WebPrefsStore;

impl PrefsStore for WebPrefsStore {
    fn load_pref<'a>(&'a self, key: &'a str) -> PrefsStoreFuture<'a, HostResult<Option<String>>> {
        Box::pin(async move { crate::bridge::load_pref(key).await })
    }

    fn save_pref<'a>(
        &'a self,
        key: &'a str,
        raw_json: &'a str,
    ) -> PrefsStoreFuture<'a, HostResult<()>> {
        Box::pin(async move { crate::bridge::save_pref(key, raw_json).await })
    }

    fn delete_pref<'a>(&'a self, key: &'a str) -> PrefsStoreFuture<'a, HostResult<()>> {
        Box::pin(async move { crate::bridge::delete_pref(key).await })
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_browser_prefs_adapter_matches_bridge_fallback_behavior() {
        let store = WebPrefsStore;
        let store_obj: &dyn PrefsStore = &store;

        assert_eq!(
            block_on(store_obj.load_pref("origin.browser.pref")).expect("load"),
            None
        );
        block_on(store_obj.save_pref("origin.browser.pref", "\"value\"")).expect("save");
        block_on(store_obj.delete_pref("origin.browser.pref")).expect("delete");
    }
}
