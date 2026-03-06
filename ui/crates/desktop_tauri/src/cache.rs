//! Typed Tauri command handlers for cache-domain text entries.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use tauri::Manager;

type CacheDomain = BTreeMap<String, String>;
type CacheMap = BTreeMap<String, CacheDomain>;

fn cache_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data dir: {err}"))?
        .join("cache");
    fs::create_dir_all(&dir).map_err(|err| format!("failed to create cache dir: {err}"))?;
    Ok(dir.join("cache.json"))
}

fn load_cache_map(path: &Path) -> Result<CacheMap, String> {
    if !path.exists() {
        return Ok(CacheMap::new());
    }
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(CacheMap::new());
    }
    serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse cache map {}: {err}", path.display()))
}

fn save_cache_map(path: &Path, map: &CacheMap) -> Result<(), String> {
    let serialized = serde_json::to_string(map)
        .map_err(|err| format!("failed to serialize cache map: {err}"))?;
    fs::write(path, serialized).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn validate_cache_key(cache_name: &str, key: &str) -> Result<(), String> {
    if cache_name.is_empty() || key.is_empty() {
        Err("cache_name and key must not be empty".to_string())
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone)]
/// Scoped cache storage service backed by a single JSON map file.
pub(crate) struct ScopedCacheStore {
    file: PathBuf,
}

impl ScopedCacheStore {
    /// Creates a scoped cache service rooted at `root`.
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref();
        fs::create_dir_all(root)
            .map_err(|err| format!("failed to create cache dir {}: {err}", root.display()))?;
        Ok(Self {
            file: root.join("cache.json"),
        })
    }

    fn from_app(app: &tauri::AppHandle) -> Result<Self, String> {
        let file = cache_file(app)?;
        let root = file
            .parent()
            .ok_or_else(|| format!("cache file {} has no parent", file.display()))?;
        Self::from_root(root)
    }

    /// Stores cache text content under `cache_name` and `key`.
    pub fn put_text(&self, cache_name: &str, key: &str, value: &str) -> Result<(), String> {
        validate_cache_key(cache_name, key)?;
        let mut map = load_cache_map(&self.file)?;
        let domain = map.entry(cache_name.to_string()).or_default();
        domain.insert(key.to_string(), value.to_string());
        save_cache_map(&self.file, &map)
    }

    /// Loads cache text content by `cache_name` and `key`.
    pub fn get_text(&self, cache_name: &str, key: &str) -> Result<Option<String>, String> {
        validate_cache_key(cache_name, key)?;
        let map = load_cache_map(&self.file)?;
        Ok(map
            .get(cache_name)
            .and_then(|domain| domain.get(key))
            .cloned())
    }

    /// Deletes cache content by `cache_name` and `key`.
    pub fn delete(&self, cache_name: &str, key: &str) -> Result<(), String> {
        validate_cache_key(cache_name, key)?;
        let mut map = load_cache_map(&self.file)?;
        if let Some(domain) = map.get_mut(cache_name) {
            domain.remove(key);
            if domain.is_empty() {
                map.remove(cache_name);
            }
        }
        save_cache_map(&self.file, &map)
    }
}

/// Stores cache text content under `cache_name` and `key`.
#[tauri::command]
pub fn cache_put_text(
    app: tauri::AppHandle,
    cache_name: String,
    key: String,
    value: String,
) -> Result<(), String> {
    ScopedCacheStore::from_app(&app)?.put_text(&cache_name, &key, &value)
}

/// Loads cache text content by `cache_name` and `key`.
#[tauri::command]
pub fn cache_get_text(
    app: tauri::AppHandle,
    cache_name: String,
    key: String,
) -> Result<Option<String>, String> {
    ScopedCacheStore::from_app(&app)?.get_text(&cache_name, &key)
}

/// Deletes cache content by `cache_name` and `key`.
#[tauri::command]
pub fn cache_delete(app: tauri::AppHandle, cache_name: String, key: String) -> Result<(), String> {
    ScopedCacheStore::from_app(&app)?.delete(&cache_name, &key)
}

#[cfg(test)]
mod tests {
    use super::{load_cache_map, save_cache_map, CacheMap, ScopedCacheStore};
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "desktop_tauri_cache_{}_{}.json",
            process::id(),
            now
        ))
    }

    fn temp_dir_path() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("desktop_tauri_cache_dir_{}_{}", process::id(), now));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn cache_map_round_trips() {
        let path = temp_file_path();
        let _ = fs::remove_file(&path);

        let initial = load_cache_map(&path).expect("load should succeed when file is missing");
        assert!(initial.is_empty());

        let mut map = CacheMap::new();
        map.entry("retrodesk-explorer-cache-v1".to_string())
            .or_default()
            .insert(
                "file-preview:/docs/readme.txt".to_string(),
                "hello".to_string(),
            );
        save_cache_map(&path, &map).expect("save map");
        let loaded = load_cache_map(&path).expect("reload map");
        assert_eq!(loaded, map);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn scoped_cache_store_rejects_empty_cache_name_or_key() {
        let root = temp_dir_path();
        let store = ScopedCacheStore::from_root(&root).expect("init scoped cache store");

        let cases = [("", "k"), ("cache", ""), ("", "")];
        for (cache_name, key) in cases {
            let expected = "cache_name and key must not be empty";
            let put_err = store
                .put_text(cache_name, key, "value")
                .expect_err("empty cache_name/key put should fail");
            assert_eq!(put_err, expected);
            let get_err = store
                .get_text(cache_name, key)
                .expect_err("empty cache_name/key get should fail");
            assert_eq!(get_err, expected);
            let delete_err = store
                .delete(cache_name, key)
                .expect_err("empty cache_name/key delete should fail");
            assert_eq!(delete_err, expected);
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scoped_cache_store_reports_malformed_map_parse_error() {
        let root = temp_dir_path();
        let cache_path = root.join("cache.json");
        fs::write(&cache_path, "{\"bad\":").expect("write malformed cache map");
        let store = ScopedCacheStore::from_root(&root).expect("init scoped cache store");

        let err = store
            .get_text(
                "retrodesk-explorer-cache-v1",
                "file-preview:/docs/readme.txt",
            )
            .expect_err("malformed cache map should fail");
        assert!(
            err.starts_with(&format!(
                "failed to parse cache map {}:",
                cache_path.display()
            )),
            "unexpected error: {err}"
        );

        let _ = fs::remove_dir_all(root);
    }
}
