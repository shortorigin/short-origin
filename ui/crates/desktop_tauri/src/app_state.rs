//! Typed Tauri command handlers for app-state envelopes.

use std::fs;
use std::path::{Path, PathBuf};

use platform_host::AppStateEnvelope;
use tauri::Manager;

fn validate_namespace(namespace: &str) -> Result<(), String> {
    if namespace.is_empty() {
        return Err("Namespace must not be empty".to_string());
    }
    if !namespace
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        return Err(format!(
            "Namespace `{namespace}` contains unsupported characters"
        ));
    }
    Ok(())
}

fn app_state_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data dir: {err}"))?
        .join("app_state");
    fs::create_dir_all(&root).map_err(|err| format!("failed to create app-state dir: {err}"))?;
    Ok(root)
}

fn namespace_file(root: &Path, namespace: &str) -> Result<PathBuf, String> {
    validate_namespace(namespace)?;
    Ok(root.join(format!("{namespace}.json")))
}

fn parse_envelope(path: &Path, raw: &str) -> Result<AppStateEnvelope, String> {
    serde_json::from_str(raw).map_err(|err| {
        format!(
            "failed to parse app-state envelope {}: {err}",
            path.display()
        )
    })
}

fn namespace_from_file_name(path: &Path) -> Option<String> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
        return None;
    }
    let stem = path.file_stem()?.to_string_lossy().to_string();
    if validate_namespace(&stem).is_ok() {
        Some(stem)
    } else {
        None
    }
}

#[derive(Debug, Clone)]
/// Scoped app-state storage service rooted at a native directory.
pub(crate) struct ScopedAppStateStore {
    root: PathBuf,
}

impl ScopedAppStateStore {
    /// Creates a scoped app-state store rooted at `root`.
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref();
        fs::create_dir_all(root)
            .map_err(|err| format!("failed to create app-state dir {}: {err}", root.display()))?;
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    fn from_app(app: &tauri::AppHandle) -> Result<Self, String> {
        Self::from_root(app_state_root(app)?)
    }

    /// Loads a typed app-state envelope by namespace.
    pub fn load(&self, namespace: &str) -> Result<Option<AppStateEnvelope>, String> {
        let path = namespace_file(&self.root, namespace)?;
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        let envelope = parse_envelope(&path, &raw)?;
        Ok(Some(envelope))
    }

    /// Saves an app-state envelope with monotonic timestamp semantics.
    pub fn save(&self, envelope: &AppStateEnvelope) -> Result<(), String> {
        validate_namespace(&envelope.namespace)?;
        let path = namespace_file(&self.root, &envelope.namespace)?;

        if path.exists() {
            let raw = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read existing {}: {err}", path.display()))?;
            if let Ok(existing) = parse_envelope(&path, &raw) {
                if existing.updated_at_unix_ms >= envelope.updated_at_unix_ms {
                    return Ok(());
                }
            }
        }

        let serialized = serde_json::to_string(envelope)
            .map_err(|err| format!("failed to serialize app-state envelope: {err}"))?;
        fs::write(&path, serialized)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))
    }

    /// Deletes persisted app-state for a namespace.
    pub fn delete(&self, namespace: &str) -> Result<(), String> {
        let path = namespace_file(&self.root, namespace)?;
        if !path.exists() {
            return Ok(());
        }
        fs::remove_file(&path).map_err(|err| format!("failed to delete {}: {err}", path.display()))
    }

    /// Lists namespaces currently present in storage.
    pub fn namespaces(&self) -> Result<Vec<String>, String> {
        let entries = fs::read_dir(&self.root)
            .map_err(|err| format!("failed to read {}: {err}", self.root.display()))?;

        let mut namespaces = Vec::new();
        for entry in entries {
            let path = entry
                .map_err(|err| format!("failed to read app-state dir entry: {err}"))?
                .path();
            if let Some(namespace) = namespace_from_file_name(&path) {
                namespaces.push(namespace);
            }
        }
        namespaces.sort();
        namespaces.dedup();
        Ok(namespaces)
    }
}

/// Loads a typed app-state envelope by namespace from the native app-data directory.
#[tauri::command]
pub fn app_state_load(
    app: tauri::AppHandle,
    namespace: String,
) -> Result<Option<AppStateEnvelope>, String> {
    ScopedAppStateStore::from_app(&app)?.load(&namespace)
}

/// Saves a typed app-state envelope in the native app-data directory.
///
/// Save behavior mirrors the browser bridge monotonic update rule:
/// if an existing envelope has `updated_at_unix_ms >= incoming.updated_at_unix_ms`,
/// the write is treated as a no-op.
#[tauri::command]
pub fn app_state_save(app: tauri::AppHandle, envelope: AppStateEnvelope) -> Result<(), String> {
    ScopedAppStateStore::from_app(&app)?.save(&envelope)
}

/// Deletes persisted app-state for a namespace.
#[tauri::command]
pub fn app_state_delete(app: tauri::AppHandle, namespace: String) -> Result<(), String> {
    ScopedAppStateStore::from_app(&app)?.delete(&namespace)
}

/// Lists namespaces currently present in native app-state storage.
#[tauri::command]
pub fn app_state_namespaces(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    ScopedAppStateStore::from_app(&app)?.namespaces()
}

#[cfg(test)]
mod tests {
    use super::{namespace_from_file_name, validate_namespace, ScopedAppStateStore};
    use platform_host::{AppStateEnvelope, APP_STATE_ENVELOPE_VERSION};
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir_path() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("desktop_tauri_app_state_{}_{}", process::id(), now));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn validates_namespace_character_policy() {
        assert!(validate_namespace("system.desktop").is_ok());
        assert!(validate_namespace("app_calculator").is_ok());
        assert!(validate_namespace("app-notepad").is_ok());
        assert!(validate_namespace("app/notepad").is_err());
        assert!(validate_namespace("../escape").is_err());
        assert!(validate_namespace("").is_err());
    }

    #[test]
    fn parses_namespace_from_json_file_name() {
        assert_eq!(
            namespace_from_file_name(Path::new("system.desktop.json")),
            Some("system.desktop".to_string())
        );
        assert_eq!(namespace_from_file_name(Path::new("bad?.json")), None);
        assert_eq!(namespace_from_file_name(Path::new("readme.md")), None);
    }

    #[test]
    fn app_state_store_rejects_invalid_namespaces_with_deterministic_errors() {
        let root = temp_dir_path();
        let store = ScopedAppStateStore::from_root(&root).expect("init scoped app-state store");
        let cases = [
            ("", "Namespace must not be empty"),
            (
                "../escape",
                "Namespace `../escape` contains unsupported characters",
            ),
            (
                "bad?name",
                "Namespace `bad?name` contains unsupported characters",
            ),
        ];

        for (namespace, expected) in cases {
            let load_err = store
                .load(namespace)
                .expect_err("invalid namespace load must fail");
            assert_eq!(load_err, expected);

            let delete_err = store
                .delete(namespace)
                .expect_err("invalid namespace delete must fail");
            assert_eq!(delete_err, expected);

            let save_err = store
                .save(&AppStateEnvelope {
                    envelope_version: APP_STATE_ENVELOPE_VERSION,
                    namespace: namespace.to_string(),
                    schema_version: 1,
                    updated_at_unix_ms: 10,
                    payload: json!({"ok": true}),
                })
                .expect_err("invalid namespace save must fail");
            assert_eq!(save_err, expected);
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn app_state_store_reports_parse_failure_for_malformed_envelope() {
        let root = temp_dir_path();
        let store = ScopedAppStateStore::from_root(&root).expect("init scoped app-state store");
        let bad_file = root.join("app.bad.json");
        fs::write(&bad_file, "{\"envelope_version\":").expect("write malformed app-state envelope");

        let err = store
            .load("app.bad")
            .expect_err("malformed envelope should fail to parse");
        assert!(
            err.starts_with(&format!(
                "failed to parse app-state envelope {}:",
                bad_file.display()
            )),
            "unexpected error: {err}"
        );

        let _ = fs::remove_dir_all(root);
    }
}
