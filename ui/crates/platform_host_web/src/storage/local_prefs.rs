//! `localStorage`-backed preference store implementation.
//!
//! This adapter is intentionally small and synchronous at the browser API boundary, while also
//! implementing [`platform_host::PrefsStore`] (async trait) for compatibility with higher-level
//! host abstractions.

use platform_host::{PrefsStore, PrefsStoreFuture};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone, Copy, Default)]
/// Browser preference store backed by `window.localStorage`.
pub struct WebPrefsStore;

impl WebPrefsStore {
    /// Loads a raw JSON string for a preference key.
    pub fn load_json(self, key: &str) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            let storage = web_sys::window()?.local_storage().ok().flatten()?;
            storage.get_item(key).ok().flatten()
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            None
        }
    }

    /// Saves a raw JSON string for a preference key.
    ///
    /// # Errors
    ///
    /// Returns an error when localStorage is unavailable or the write fails.
    pub fn save_json(self, key: &str, raw_json: &str) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            let storage = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .ok_or_else(|| "localStorage unavailable".to_string())?;
            storage
                .set_item(key, raw_json)
                .map_err(|e| format!("localStorage set_item failed: {e:?}"))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (key, raw_json);
            Ok(())
        }
    }

    /// Deletes a preference key from localStorage.
    ///
    /// # Errors
    ///
    /// Returns an error when localStorage is unavailable or the delete fails.
    pub fn delete_json(self, key: &str) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            let storage = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .ok_or_else(|| "localStorage unavailable".to_string())?;
            storage
                .remove_item(key)
                .map_err(|e| format!("localStorage remove_item failed: {e:?}"))?;
            Ok(())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            Ok(())
        }
    }

    /// Loads and deserializes a typed preference value.
    pub fn load_typed<T: DeserializeOwned>(self, key: &str) -> Option<T> {
        let raw = self.load_json(key)?;
        serde_json::from_str(&raw).ok()
    }

    /// Serializes and saves a typed preference value.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization or localStorage write fails.
    pub fn save_typed<T: Serialize>(self, key: &str, value: &T) -> Result<(), String> {
        let raw = serde_json::to_string(value).map_err(|e| e.to_string())?;
        self.save_json(key, &raw)
    }
}

impl PrefsStore for WebPrefsStore {
    fn load_pref<'a>(
        &'a self,
        key: &'a str,
    ) -> PrefsStoreFuture<'a, Result<Option<String>, String>> {
        let store = *self;
        Box::pin(async move { Ok(store.load_json(key)) })
    }

    fn save_pref<'a>(
        &'a self,
        key: &'a str,
        raw_json: &'a str,
    ) -> PrefsStoreFuture<'a, Result<(), String>> {
        let store = *self;
        Box::pin(async move { store.save_json(key, raw_json) })
    }

    fn delete_pref<'a>(&'a self, key: &'a str) -> PrefsStoreFuture<'a, Result<(), String>> {
        let store = *self;
        Box::pin(async move { store.delete_json(key) })
    }
}
