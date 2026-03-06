//! Optional terminal process/PTY host contract for desktop-native shell execution.

use std::{future::Future, pin::Pin};

/// Stable identifier for one host terminal-process session.
pub type TerminalSessionId = u64;

/// Stream event emitted by a host terminal-process backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEvent {
    /// UTF-8 text emitted by the running process or PTY.
    Output {
        /// UTF-8 payload emitted by the running session.
        text: String,
    },
    /// Process/session exited with an optional exit code.
    Exit {
        /// Exit status code when the host provides one.
        code: Option<i32>,
    },
}

/// Input written to a running host terminal-process session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalWriteRequest {
    /// Target session identifier.
    pub session_id: TerminalSessionId,
    /// Text to write.
    pub text: String,
}

/// Resize request for a running host terminal-process session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalResizeRequest {
    /// Target session identifier.
    pub session_id: TerminalSessionId,
    /// New terminal column count.
    pub cols: u16,
    /// New terminal row count.
    pub rows: u16,
}

/// Object-safe boxed future used by [`TerminalProcessService`] async methods.
pub type TerminalProcessFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Optional host service for native process/PTY-backed terminal sessions.
pub trait TerminalProcessService {
    /// Spawns a new terminal-process session.
    fn spawn<'a>(
        &'a self,
        cwd: &'a str,
    ) -> TerminalProcessFuture<'a, Result<TerminalSessionId, String>>;

    /// Writes text into a running terminal-process session.
    fn write<'a>(
        &'a self,
        request: TerminalWriteRequest,
    ) -> TerminalProcessFuture<'a, Result<(), String>>;

    /// Resizes a running terminal-process session.
    fn resize<'a>(
        &'a self,
        request: TerminalResizeRequest,
    ) -> TerminalProcessFuture<'a, Result<(), String>>;

    /// Cancels a running terminal-process session.
    fn cancel<'a>(
        &'a self,
        session_id: TerminalSessionId,
    ) -> TerminalProcessFuture<'a, Result<(), String>>;
}

#[derive(Debug, Clone, Copy, Default)]
/// No-op terminal-process backend used until desktop-native PTY support lands.
pub struct NoopTerminalProcessService;

impl TerminalProcessService for NoopTerminalProcessService {
    fn spawn<'a>(
        &'a self,
        _cwd: &'a str,
    ) -> TerminalProcessFuture<'a, Result<TerminalSessionId, String>> {
        Box::pin(async { Err("terminal process unavailable: spawn".to_string()) })
    }

    fn write<'a>(
        &'a self,
        _request: TerminalWriteRequest,
    ) -> TerminalProcessFuture<'a, Result<(), String>> {
        Box::pin(async { Err("terminal process unavailable: write".to_string()) })
    }

    fn resize<'a>(
        &'a self,
        _request: TerminalResizeRequest,
    ) -> TerminalProcessFuture<'a, Result<(), String>> {
        Box::pin(async { Err("terminal process unavailable: resize".to_string()) })
    }

    fn cancel<'a>(
        &'a self,
        _session_id: TerminalSessionId,
    ) -> TerminalProcessFuture<'a, Result<(), String>> {
        Box::pin(async { Err("terminal process unavailable: cancel".to_string()) })
    }
}
