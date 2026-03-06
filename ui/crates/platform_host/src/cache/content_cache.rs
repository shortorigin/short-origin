//! Content cache service contracts and adapters.

use std::{cell::RefCell, collections::HashMap, future::Future, pin::Pin, rc::Rc};

use serde::{de::DeserializeOwned, Serialize};

/// Object-safe boxed future used by [`ContentCache`] async methods.
pub type ContentCacheFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Host cache service for string content keyed by cache-name and key.
pub trait ContentCache {
    /// Stores text content under `cache_name` and `key`.
    fn put_text<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
        value: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>>;

    /// Reads text content by `cache_name` and `key`.
    fn get_text<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
    ) -> ContentCacheFuture<'a, Result<Option<String>, String>>;

    /// Deletes cached content by `cache_name` and `key`.
    fn delete<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>>;
}

#[derive(Debug, Clone, Copy, Default)]
/// No-op cache adapter for unsupported targets and baseline tests.
pub struct NoopContentCache;

impl ContentCache for NoopContentCache {
    fn put_text<'a>(
        &'a self,
        _cache_name: &'a str,
        _key: &'a str,
        _value: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }

    fn get_text<'a>(
        &'a self,
        _cache_name: &'a str,
        _key: &'a str,
    ) -> ContentCacheFuture<'a, Result<Option<String>, String>> {
        Box::pin(async { Ok(None) })
    }

    fn delete<'a>(
        &'a self,
        _cache_name: &'a str,
        _key: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

#[derive(Debug, Clone, Default)]
/// In-memory cache adapter keyed by `(cache_name, key)` tuples.
pub struct MemoryContentCache {
    inner: Rc<RefCell<HashMap<(String, String), String>>>,
}

impl ContentCache for MemoryContentCache {
    fn put_text<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
        value: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>> {
        Box::pin(async move {
            self.inner
                .borrow_mut()
                .insert((cache_name.to_string(), key.to_string()), value.to_string());
            Ok(())
        })
    }

    fn get_text<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
    ) -> ContentCacheFuture<'a, Result<Option<String>, String>> {
        Box::pin(async move {
            Ok(self
                .inner
                .borrow()
                .get(&(cache_name.to_string(), key.to_string()))
                .cloned())
        })
    }

    fn delete<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>> {
        Box::pin(async move {
            self.inner
                .borrow_mut()
                .remove(&(cache_name.to_string(), key.to_string()));
            Ok(())
        })
    }
}

/// Serializes and stores a JSON value through a [`ContentCache`] implementation.
///
/// # Errors
///
/// Returns an error when serialization or cache storage fails.
pub async fn cache_put_json_with<C: ContentCache + ?Sized, T: Serialize>(
    cache: &C,
    cache_name: &str,
    key: &str,
    value: &T,
) -> Result<(), String> {
    let raw = serde_json::to_string(value).map_err(|e| e.to_string())?;
    cache.put_text(cache_name, key, &raw).await
}

/// Reads and deserializes a JSON value through a [`ContentCache`] implementation.
///
/// # Errors
///
/// Returns an error when cache access or JSON deserialization fails.
pub async fn cache_get_json_with<C: ContentCache + ?Sized, T: DeserializeOwned>(
    cache: &C,
    cache_name: &str,
    key: &str,
) -> Result<Option<T>, String> {
    let Some(raw) = cache.get_text(cache_name, key).await? else {
        return Ok(None);
    };
    let value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct CachedThing {
        answer: u32,
    }

    #[test]
    fn memory_content_cache_put_get_delete_round_trip() {
        let cache = MemoryContentCache::default();
        let cache_obj: &dyn ContentCache = &cache;

        block_on(cache_obj.put_text("preview", "/file.txt", "hello")).expect("put");
        assert_eq!(
            block_on(cache_obj.get_text("preview", "/file.txt")).expect("get"),
            Some("hello".to_string())
        );
        block_on(cache_obj.delete("preview", "/file.txt")).expect("delete");
        assert_eq!(
            block_on(cache_obj.get_text("preview", "/file.txt")).expect("get"),
            None
        );
    }

    #[test]
    fn cache_json_helpers_round_trip() {
        let cache = MemoryContentCache::default();
        let cache_obj: &dyn ContentCache = &cache;

        block_on(cache_put_json_with(
            cache_obj,
            "preview",
            "k",
            &CachedThing { answer: 42 },
        ))
        .expect("put json");
        let loaded: Option<CachedThing> =
            block_on(cache_get_json_with(cache_obj, "preview", "k")).expect("get json");
        assert_eq!(loaded, Some(CachedThing { answer: 42 }));
    }

    #[test]
    fn noop_content_cache_is_empty_and_successful() {
        let cache = NoopContentCache;
        let cache_obj: &dyn ContentCache = &cache;
        block_on(cache_obj.put_text("x", "y", "z")).expect("put");
        assert_eq!(block_on(cache_obj.get_text("x", "y")).expect("get"), None);
        block_on(cache_obj.delete("x", "y")).expect("delete");
    }
}
