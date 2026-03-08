//! Notification host-service adapters for browser and desktop-webview contexts.

use crate::bridge;
use platform_host::{HostResult, NotificationFuture, NotificationService};

#[derive(Debug, Clone, Copy, Default)]
/// Browser notification adapter backed by the Web Notifications API.
pub struct WebNotificationService;

impl NotificationService for WebNotificationService {
    fn notify<'a>(
        &'a self,
        title: &'a str,
        body: &'a str,
    ) -> NotificationFuture<'a, HostResult<()>> {
        Box::pin(async move {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsValue;
                let rendered = if body.trim().is_empty() {
                    title.to_string()
                } else {
                    format!("{title}: {body}")
                };
                return web_sys::Notification::new(&rendered).map(|_| ()).map_err(
                    |err: JsValue| {
                        platform_host::HostError::notification(
                            platform_host::NotificationErrorKind::Dispatch,
                            "Notification could not be delivered",
                        )
                        .with_operation("notification.notify")
                        .with_internal(format!("{err:?}"))
                    },
                );
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
    ) -> NotificationFuture<'a, HostResult<()>> {
        Box::pin(async move { bridge::send_notification(title, body).await })
    }
}
