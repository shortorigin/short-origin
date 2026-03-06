//! Typed Tauri command handlers for lightweight preference storage.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use tauri::Manager;

type PrefMap = BTreeMap<String, String>;

fn prefs_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data dir: {err}"))?
        .join("prefs");
    fs::create_dir_all(&dir).map_err(|err| format!("failed to create prefs dir: {err}"))?;
    Ok(dir.join("prefs.json"))
}

fn load_pref_map(path: &Path) -> Result<PrefMap, String> {
    if !path.exists() {
        return Ok(PrefMap::new());
    }
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(PrefMap::new());
    }
    serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse prefs map {}: {err}", path.display()))
}

fn save_pref_map(path: &Path, map: &PrefMap) -> Result<(), String> {
    let serialized = serde_json::to_string(map)
        .map_err(|err| format!("failed to serialize prefs map: {err}"))?;
    fs::write(path, serialized).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        Err("Preference key must not be empty".to_string())
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone)]
/// Scoped preference storage service backed by a single JSON map file.
pub(crate) struct ScopedPrefsStore {
    file: PathBuf,
}

impl ScopedPrefsStore {
    /// Creates a scoped prefs service rooted at `root`.
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref();
        fs::create_dir_all(root)
            .map_err(|err| format!("failed to create prefs dir {}: {err}", root.display()))?;
        Ok(Self {
            file: root.join("prefs.json"),
        })
    }

    fn from_app(app: &tauri::AppHandle) -> Result<Self, String> {
        let file = prefs_file(app)?;
        let root = file
            .parent()
            .ok_or_else(|| format!("prefs file {} has no parent", file.display()))?;
        Self::from_root(root)
    }

    /// Loads a preference payload by key.
    pub fn load(&self, key: &str) -> Result<Option<String>, String> {
        validate_key(key)?;
        let map = load_pref_map(&self.file)?;
        Ok(map.get(key).cloned())
    }

    /// Saves a preference payload by key.
    pub fn save(&self, key: &str, raw_json: &str) -> Result<(), String> {
        validate_key(key)?;
        let mut map = load_pref_map(&self.file)?;
        map.insert(key.to_string(), raw_json.to_string());
        save_pref_map(&self.file, &map)
    }

    /// Deletes a preference key.
    pub fn delete(&self, key: &str) -> Result<(), String> {
        validate_key(key)?;
        let mut map = load_pref_map(&self.file)?;
        map.remove(key);
        save_pref_map(&self.file, &map)
    }
}

/// Loads a preference raw JSON payload by key.
#[tauri::command]
pub fn prefs_load(app: tauri::AppHandle, key: String) -> Result<Option<String>, String> {
    ScopedPrefsStore::from_app(&app)?.load(&key)
}

/// Saves a preference raw JSON payload by key.
#[tauri::command]
pub fn prefs_save(app: tauri::AppHandle, key: String, raw_json: String) -> Result<(), String> {
    ScopedPrefsStore::from_app(&app)?.save(&key, &raw_json)
}

/// Deletes a preference key.
#[tauri::command]
pub fn prefs_delete(app: tauri::AppHandle, key: String) -> Result<(), String> {
    ScopedPrefsStore::from_app(&app)?.delete(&key)
}

#[cfg(test)]
mod tests {
    use super::{load_pref_map, save_pref_map, PrefMap, ScopedPrefsStore};
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
            "desktop_tauri_prefs_{}_{}.json",
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
            std::env::temp_dir().join(format!("desktop_tauri_prefs_dir_{}_{}", process::id(), now));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn pref_map_round_trips() {
        let path = temp_file_path();
        let _ = fs::remove_file(&path);

        let initial = load_pref_map(&path).expect("load should succeed when file is missing");
        assert!(initial.is_empty());

        let mut map = PrefMap::new();
        map.insert(
            "retrodesk.explorer.prefs.v1".to_string(),
            "{\"k\":1}".to_string(),
        );
        save_pref_map(&path, &map).expect("save map");
        let loaded = load_pref_map(&path).expect("reload map");
        assert_eq!(loaded, map);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn scoped_prefs_store_rejects_empty_key_for_all_operations() {
        let root = temp_dir_path();
        let store = ScopedPrefsStore::from_root(&root).expect("init scoped prefs store");

        let expected = "Preference key must not be empty";
        let load_err = store.load("").expect_err("empty key load should fail");
        assert_eq!(load_err, expected);
        let save_err = store
            .save("", "{\"a\":1}")
            .expect_err("empty key save should fail");
        assert_eq!(save_err, expected);
        let delete_err = store.delete("").expect_err("empty key delete should fail");
        assert_eq!(delete_err, expected);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scoped_prefs_store_reports_malformed_map_parse_error() {
        let root = temp_dir_path();
        let prefs_path = root.join("prefs.json");
        fs::write(&prefs_path, "{\"bad\":").expect("write malformed prefs map");
        let store = ScopedPrefsStore::from_root(&root).expect("init scoped prefs store");

        let err = store
            .load("retrodesk.explorer.prefs.v1")
            .expect_err("malformed prefs map should fail");
        assert!(
            err.starts_with(&format!(
                "failed to parse prefs map {}:",
                prefs_path.display()
            )),
            "unexpected error: {err}"
        );

        let _ = fs::remove_dir_all(root);
    }
}
