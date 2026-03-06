//! In-memory session-scoped storage primitives.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
/// In-memory session-scoped key/value JSON store used by non-durable UI state.
pub struct MemorySessionStore {
    inner: Rc<RefCell<HashMap<String, Value>>>,
}

impl Default for MemorySessionStore {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl MemorySessionStore {
    /// Stores a raw JSON value by key.
    pub fn set_json(&self, key: impl Into<String>, value: Value) {
        self.inner.borrow_mut().insert(key.into(), value);
    }

    /// Reads a raw JSON value by key.
    pub fn get_json(&self, key: &str) -> Option<Value> {
        self.inner.borrow().get(key).cloned()
    }

    /// Removes a value by key.
    pub fn remove(&self, key: &str) {
        self.inner.borrow_mut().remove(key);
    }

    /// Serializes and stores a typed value.
    ///
    /// # Errors
    ///
    /// Returns an error when `value` cannot be serialized to JSON.
    pub fn set<T: Serialize>(&self, key: impl Into<String>, value: &T) -> Result<(), String> {
        let json = serde_json::to_value(value).map_err(|e| e.to_string())?;
        self.set_json(key, json);
        Ok(())
    }

    /// Reads and deserializes a typed value.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.get_json(key)
            .and_then(|value| serde_json::from_value(value).ok())
    }
}

thread_local! {
    static GLOBAL_SESSION_STORE: MemorySessionStore = MemorySessionStore::default();
}

/// Returns the process-local session store instance.
pub fn session_store() -> MemorySessionStore {
    GLOBAL_SESSION_STORE.with(|store| store.clone())
}
