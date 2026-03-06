//! Lightweight preference storage contracts and adapters.

use std::{cell::RefCell, collections::HashMap, future::Future, pin::Pin, rc::Rc};

use serde::{de::DeserializeOwned, Serialize};

/// Object-safe boxed future used by [`PrefsStore`] async methods.
pub type PrefsStoreFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Host service for lightweight preference values (JSON stored as text per key).
pub trait PrefsStore {
    /// Loads a raw JSON string for a preference key.
    fn load_pref<'a>(
        &'a self,
        key: &'a str,
    ) -> PrefsStoreFuture<'a, Result<Option<String>, String>>;

    /// Saves a raw JSON string for a preference key.
    fn save_pref<'a>(
        &'a self,
        key: &'a str,
        raw_json: &'a str,
    ) -> PrefsStoreFuture<'a, Result<(), String>>;

    /// Deletes a preference key.
    fn delete_pref<'a>(&'a self, key: &'a str) -> PrefsStoreFuture<'a, Result<(), String>>;
}

#[derive(Debug, Clone, Copy, Default)]
/// No-op preference store for unsupported targets and baseline tests.
pub struct NoopPrefsStore;

impl PrefsStore for NoopPrefsStore {
    fn load_pref<'a>(
        &'a self,
        _key: &'a str,
    ) -> PrefsStoreFuture<'a, Result<Option<String>, String>> {
        Box::pin(async { Ok(None) })
    }

    fn save_pref<'a>(
        &'a self,
        _key: &'a str,
        _raw_json: &'a str,
    ) -> PrefsStoreFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }

    fn delete_pref<'a>(&'a self, _key: &'a str) -> PrefsStoreFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

#[derive(Debug, Clone, Default)]
/// In-memory preference store keyed by string.
pub struct MemoryPrefsStore {
    inner: Rc<RefCell<HashMap<String, String>>>,
}

impl PrefsStore for MemoryPrefsStore {
    fn load_pref<'a>(
        &'a self,
        key: &'a str,
    ) -> PrefsStoreFuture<'a, Result<Option<String>, String>> {
        Box::pin(async move { Ok(self.inner.borrow().get(key).cloned()) })
    }

    fn save_pref<'a>(
        &'a self,
        key: &'a str,
        raw_json: &'a str,
    ) -> PrefsStoreFuture<'a, Result<(), String>> {
        Box::pin(async move {
            self.inner
                .borrow_mut()
                .insert(key.to_string(), raw_json.to_string());
            Ok(())
        })
    }

    fn delete_pref<'a>(&'a self, key: &'a str) -> PrefsStoreFuture<'a, Result<(), String>> {
        Box::pin(async move {
            self.inner.borrow_mut().remove(key);
            Ok(())
        })
    }
}

/// Loads and deserializes a typed preference value through a [`PrefsStore`] implementation.
///
/// # Errors
///
/// Returns an error when the store or JSON deserialization fails.
pub async fn load_pref_with<S: PrefsStore + ?Sized, T: DeserializeOwned>(
    store: &S,
    key: &str,
) -> Result<Option<T>, String> {
    let Some(raw) = store.load_pref(key).await? else {
        return Ok(None);
    };
    let value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(Some(value))
}

/// Serializes and saves a typed preference value through a [`PrefsStore`] implementation.
///
/// # Errors
///
/// Returns an error when serialization or store save fails.
pub async fn save_pref_with<S: PrefsStore + ?Sized, T: Serialize>(
    store: &S,
    key: &str,
    value: &T,
) -> Result<(), String> {
    let raw = serde_json::to_string(value).map_err(|e| e.to_string())?;
    store.save_pref(key, &raw).await
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct PrefThing {
        details_visible: bool,
    }

    #[test]
    fn memory_prefs_store_round_trip_and_delete() {
        let store = MemoryPrefsStore::default();
        let store_obj: &dyn PrefsStore = &store;

        block_on(store_obj.save_pref("pref.key", "{\"k\":1}")).expect("save");
        assert_eq!(
            block_on(store_obj.load_pref("pref.key")).expect("load"),
            Some("{\"k\":1}".to_string())
        );
        block_on(store_obj.delete_pref("pref.key")).expect("delete");
        assert_eq!(
            block_on(store_obj.load_pref("pref.key")).expect("load"),
            None
        );
    }

    #[test]
    fn typed_pref_helpers_round_trip() {
        let store = MemoryPrefsStore::default();
        let store_obj: &dyn PrefsStore = &store;
        block_on(save_pref_with(
            store_obj,
            "explorer",
            &PrefThing {
                details_visible: true,
            },
        ))
        .expect("save typed pref");

        let loaded: Option<PrefThing> =
            block_on(load_pref_with(store_obj, "explorer")).expect("load typed pref");
        assert_eq!(
            loaded,
            Some(PrefThing {
                details_visible: true
            })
        );
    }

    #[test]
    fn noop_prefs_store_is_empty_and_successful() {
        let store = NoopPrefsStore;
        let store_obj: &dyn PrefsStore = &store;
        assert_eq!(block_on(store_obj.load_pref("k")).expect("load"), None);
        block_on(store_obj.save_pref("k", "{}")).expect("save");
        block_on(store_obj.delete_pref("k")).expect("delete");
    }
}
