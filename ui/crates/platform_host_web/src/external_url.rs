//! External URL host-service adapters for browser and desktop-webview contexts.

use platform_host::{ExternalUrlFuture, ExternalUrlService};

use crate::bridge;

#[derive(Debug, Clone, Copy, Default)]
/// Browser external URL adapter backed by the bridge interop layer.
pub struct WebExternalUrlService;

impl ExternalUrlService for WebExternalUrlService {
    fn open_url<'a>(&'a self, url: &'a str) -> ExternalUrlFuture<'a, Result<(), String>> {
        Box::pin(async move { bridge::open_external_url(url).await })
    }
}

#[derive(Debug, Clone, Copy, Default)]
/// Desktop-webview external URL adapter backed by the bridge interop layer.
pub struct TauriExternalUrlService;

impl ExternalUrlService for TauriExternalUrlService {
    fn open_url<'a>(&'a self, url: &'a str) -> ExternalUrlFuture<'a, Result<(), String>> {
        Box::pin(async move { bridge::open_external_url(url).await })
    }
}
