//! Cache-domain contracts and lightweight test adapters.

mod content_cache;

pub use content_cache::{
    ContentCache, ContentCacheFuture, MemoryContentCache, NoopContentCache, cache_get_json_with,
    cache_put_json_with,
};
