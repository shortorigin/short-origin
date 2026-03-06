//! External URL host-service contracts.

use std::{future::Future, pin::Pin};

/// Object-safe boxed future used by [`ExternalUrlService`].
pub type ExternalUrlFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Host service for opening external URLs outside the desktop shell.
pub trait ExternalUrlService {
    /// Opens a URL using the host's external navigation mechanism.
    fn open_url<'a>(&'a self, url: &'a str) -> ExternalUrlFuture<'a, Result<(), String>>;
}

#[derive(Debug, Clone, Copy, Default)]
/// No-op external URL service for unsupported targets.
pub struct NoopExternalUrlService;

impl ExternalUrlService for NoopExternalUrlService {
    fn open_url<'a>(&'a self, _url: &'a str) -> ExternalUrlFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}
