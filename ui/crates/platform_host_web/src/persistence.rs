//! Browser persistence capability helpers.

/// Returns whether IndexedDB is available in the active browser context.
pub fn indexed_db_supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| js_sys::Reflect::has(window.as_ref(), &"indexedDB".into()).ok())
            .unwrap_or(false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Returns whether OPFS is available.
pub fn opfs_supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return false;
        };
        let Ok(storage) = js_sys::Reflect::get(window.navigator().as_ref(), &"storage".into())
        else {
            return false;
        };
        if storage.is_undefined() || storage.is_null() {
            return false;
        }
        js_sys::Reflect::get(&storage, &"getDirectory".into()).is_ok()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}
