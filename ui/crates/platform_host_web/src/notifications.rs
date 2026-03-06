//! Notification host-service adapters for browser and desktop-webview contexts.

use platform_host::{NotificationFuture, NotificationService};

#[derive(Debug, Clone, Copy, Default)]
/// Browser notification adapter backed by the Web Notifications API.
pub struct WebNotificationService;

impl NotificationService for WebNotificationService {
    fn notify<'a>(
        &'a self,
        title: &'a str,
        body: &'a str,
    ) -> NotificationFuture<'a, Result<(), String>> {
        Box::pin(async move {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsValue;
                let rendered = if body.trim().is_empty() {
                    title.to_string()
                } else {
                    format!("{title}: {body}")
                };
                return web_sys::Notification::new(&rendered)
                    .map(|_| ())
                    .map_err(|err: JsValue| format!("notification dispatch failed: {err:?}"));
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (title, body);
                Ok(())
            }
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
/// Desktop-webview notification adapter (currently same behavior as browser adapter).
pub struct TauriNotificationService;

impl NotificationService for TauriNotificationService {
    fn notify<'a>(
        &'a self,
        title: &'a str,
        body: &'a str,
    ) -> NotificationFuture<'a, Result<(), String>> {
        WebNotificationService.notify(title, body)
    }
}
