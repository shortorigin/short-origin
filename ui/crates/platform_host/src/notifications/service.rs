//! Notification service contracts and no-op adapter.

use std::{future::Future, pin::Pin};

/// Object-safe boxed future used by [`NotificationService`].
pub type NotificationFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Host service for user-visible notifications.
pub trait NotificationService {
    /// Dispatches a notification message.
    fn notify<'a>(
        &'a self,
        title: &'a str,
        body: &'a str,
    ) -> NotificationFuture<'a, Result<(), String>>;
}

#[derive(Debug, Clone, Copy, Default)]
/// No-op notification service for unsupported targets.
pub struct NoopNotificationService;

impl NotificationService for NoopNotificationService {
    fn notify<'a>(
        &'a self,
        _title: &'a str,
        _body: &'a str,
    ) -> NotificationFuture<'a, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}
