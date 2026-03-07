//! Browser PWA/runtime enhancement helpers.

#[cfg(target_arch = "wasm32")]
use js_sys::Function;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
const SERVICE_WORKER_ASSET: &str = "sw.js";

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn public_asset_path(base_path: &str, asset: &str) -> String {
    let asset = asset.trim_start_matches('/');
    let base_path = base_path.trim();
    if base_path.is_empty() || base_path == "/" {
        format!("/{asset}")
    } else {
        format!("{}/{}", base_path.trim_end_matches('/'), asset)
    }
}

#[cfg(target_arch = "wasm32")]
fn current_public_base_path() -> Option<String> {
    let document = web_sys::window()?.document()?;
    let base_uri = document.base_uri().ok().flatten()?;
    let url = web_sys::Url::new(&base_uri).ok()?;
    Some(url.pathname())
}

#[cfg(target_arch = "wasm32")]
fn current_hostname() -> Option<String> {
    web_sys::window()?.location().hostname().ok()
}

#[cfg(target_arch = "wasm32")]
fn is_local_dev_host() -> bool {
    matches!(
        current_hostname().as_deref(),
        Some("localhost") | Some("127.0.0.1") | Some("0.0.0.0")
    )
}

#[cfg(target_arch = "wasm32")]
fn unregister_service_workers(service_worker: JsValue) {
    leptos::spawn_local(async move {
        let Ok(get_registrations) = js_sys::Reflect::get(
            &service_worker,
            &wasm_bindgen::JsValue::from_str("getRegistrations"),
        ) else {
            return;
        };
        let Some(get_registrations_fn) = get_registrations.dyn_ref::<Function>() else {
            return;
        };
        let Ok(registrations_promise) = get_registrations_fn.call0(&service_worker) else {
            return;
        };
        let Ok(registrations_value) =
            JsFuture::from(js_sys::Promise::from(registrations_promise)).await
        else {
            return;
        };
        let registrations = js_sys::Array::from(&registrations_value);
        for registration in registrations.iter() {
            let Ok(unregister) = js_sys::Reflect::get(
                &registration,
                &wasm_bindgen::JsValue::from_str("unregister"),
            ) else {
                continue;
            };
            let Some(unregister_fn) = unregister.dyn_ref::<Function>() else {
                continue;
            };
            let _ = unregister_fn.call0(&registration);
        }
    });
}

/// Registers the service worker when the platform supports it.
pub fn register_service_worker() {
    #[cfg(target_arch = "wasm32")]
    {
        if !platform_host_web::pwa::service_worker_supported() {
            return;
        }

        let Some(window) = web_sys::window() else {
            return;
        };
        let navigator = window.navigator();
        let Ok(service_worker) = js_sys::Reflect::get(
            navigator.as_ref(),
            &wasm_bindgen::JsValue::from_str("serviceWorker"),
        ) else {
            return;
        };
        let Ok(register) = js_sys::Reflect::get(
            &service_worker,
            &wasm_bindgen::JsValue::from_str("register"),
        ) else {
            return;
        };
        if is_local_dev_host() {
            unregister_service_workers(service_worker);
            return;
        }
        let Some(register_fn) = register.dyn_ref::<js_sys::Function>() else {
            return;
        };
        let Some(base_path) = current_public_base_path() else {
            return;
        };
        let sw_path = public_asset_path(&base_path, SERVICE_WORKER_ASSET);
        let _ = register_fn.call1(&service_worker, &wasm_bindgen::JsValue::from_str(&sw_path));
    }
}

#[cfg(test)]
mod tests {
    use super::public_asset_path;

    #[test]
    fn joins_assets_at_root() {
        assert_eq!(public_asset_path("/", "sw.js"), "/sw.js");
    }

    #[test]
    fn joins_assets_under_trunk_public_path() {
        assert_eq!(
            public_asset_path("/preview/ui/", "sw.js"),
            "/preview/ui/sw.js"
        );
    }

    #[test]
    fn trims_leading_slashes_from_asset_paths() {
        assert_eq!(
            public_asset_path("/preview/ui", "/manifest.webmanifest"),
            "/preview/ui/manifest.webmanifest"
        );
    }
}
