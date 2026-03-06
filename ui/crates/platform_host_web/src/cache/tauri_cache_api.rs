//! Tauri command-backed content cache implementation.

use platform_host::{ContentCache, ContentCacheFuture};

#[derive(Debug, Clone, Copy, Default)]
/// Desktop content cache backed by Tauri command transport.
pub struct TauriContentCache;

impl ContentCache for TauriContentCache {
    fn put_text<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
        value: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>> {
        Box::pin(async move { crate::bridge::cache_put_text(cache_name, key, value).await })
    }

    fn get_text<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
    ) -> ContentCacheFuture<'a, Result<Option<String>, String>> {
        Box::pin(async move { crate::bridge::cache_get_text(cache_name, key).await })
    }

    fn delete<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>> {
        Box::pin(async move { crate::bridge::cache_delete(cache_name, key).await })
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_tauri_cache_adapter_matches_bridge_fallback_behavior() {
        let cache = TauriContentCache;
        let cache_obj: &dyn ContentCache = &cache;

        block_on(cache_obj.put_text("cache", "k", "v")).expect("put");
        assert_eq!(
            block_on(cache_obj.get_text("cache", "k")).expect("get"),
            None
        );
        block_on(cache_obj.delete("cache", "k")).expect("delete");
    }
}
