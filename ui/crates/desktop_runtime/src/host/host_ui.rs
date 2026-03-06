#[cfg(target_arch = "wasm32")]
use desktop_app_contract::window_primary_input_dom_id;
use leptos::{logging, spawn_local};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

use crate::{
    components::DesktopRuntimeContext,
    host::DesktopHostContext,
    model::WindowRect,
    reducer::{build_open_request_from_deeplink, DesktopAction},
};

pub(super) fn open_deep_link(
    runtime: DesktopRuntimeContext,
    deep_link: crate::model::DeepLinkState,
) {
    for target in deep_link.open {
        match target {
            crate::model::DeepLinkOpenTarget::App(app_id) => {
                runtime.dispatch_action(DesktopAction::ActivateApp {
                    app_id,
                    viewport: Some(runtime.host.get_value().desktop_viewport_rect(38)),
                });
            }
            target => {
                runtime.dispatch_action(DesktopAction::OpenWindow(
                    build_open_request_from_deeplink(target),
                ));
            }
        }
    }
}

pub(super) fn focus_window_input(window_id: crate::model::WindowId) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(element) = document.get_element_by_id(&window_primary_input_dom_id(window_id.0))
        else {
            return;
        };
        let Ok(element) = element.dyn_into::<web_sys::HtmlElement>() else {
            return;
        };
        let callback = Closure::once_into_js(move || {
            let _ = element.focus();
        });
        let _ = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), 0);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = window_id;
}

pub(super) fn open_external_url(host: DesktopHostContext, url: &str) {
    let url = url.to_string();
    spawn_local(async move {
        if let Err(err) = host.external_url_service().open_url(&url).await {
            logging::warn!("open external url failed for `{url}`: {err}");
        }
    });
}

pub(super) fn notify(host: DesktopHostContext, title: String, body: String) {
    spawn_local(async move {
        if let Err(err) = host.notification_service().notify(&title, &body).await {
            logging::warn!("notification dispatch failed: {err}");
        }
    });
}

pub(super) fn desktop_viewport_rect(taskbar_height_px: i32) -> WindowRect {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let width = window
                .inner_width()
                .ok()
                .and_then(|value| value.as_f64())
                .map(|value| value as i32)
                .unwrap_or(1024);
            let height = window
                .inner_height()
                .ok()
                .and_then(|value| value.as_f64())
                .map(|value| value as i32)
                .unwrap_or(768);

            return WindowRect {
                x: 0,
                y: 0,
                w: width.max(320),
                h: (height - taskbar_height_px).max(220),
            };
        }
    }

    WindowRect {
        x: 0,
        y: 0,
        w: 1024,
        h: 768 - taskbar_height_px,
    }
}
