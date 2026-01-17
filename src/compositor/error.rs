//! Compositor-specific error types.

use thiserror::Error;

/// Errors that can occur during compositor operations.
#[derive(Error, Debug)]
pub enum CompositorError {
    /// Failed to connect to compositor socket.
    #[error("Failed to connect to compositor socket: {0}")]
    ConnectionFailed(#[source] std::io::Error),

    /// Error during IPC communication.
    #[error("IPC communication error: {0}")]
    IpcError(String),

    /// Failed to parse compositor response.
    #[error("Failed to parse compositor response: {0}")]
    ParseError(#[source] serde_json::Error),

    /// Window not found.
    #[error("Window not found: {0}")]
    WindowNotFound(String),

    /// D-Bus error (for KWin).
    #[error("D-Bus error: {0}")]
    DbusError(String),

    /// Command execution failed (for fallback methods).
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
}

impl From<std::io::Error> for CompositorError {
    fn from(err: std::io::Error) -> Self {
        CompositorError::ConnectionFailed(err)
    }
}

impl From<serde_json::Error> for CompositorError {
    fn from(err: serde_json::Error) -> Self {
        CompositorError::ParseError(err)
    }
}
