//! Domain-specific error types for zlaunch.
//!
//! This module provides structured error types for different domains of the application,
//! enabling better error handling, logging, and user feedback.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// IPC-specific errors for daemon communication.
///
/// This type is serializable for use with tarpc.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum IpcError {
    /// The daemon event channel was closed unexpectedly.
    #[error("Daemon channel closed")]
    ChannelClosed,

    /// The response channel was closed before receiving a response.
    #[error("Response channel closed")]
    ResponseClosed,

    /// The requested theme was not found.
    #[error("Theme '{0}' not found")]
    ThemeNotFound(String),

    /// A general internal error occurred.
    #[error("{0}")]
    Internal(String),
}

/// Process execution errors.
#[derive(Error, Debug)]
pub enum ProcessError {
    /// The exec command string was empty.
    #[error("Empty exec command")]
    EmptyCommand,

    /// No terminal emulator could be found.
    #[error("No terminal emulator found. Set $TERMINAL environment variable.")]
    NoTerminal,

    /// Failed to spawn the process.
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[source] std::io::Error),
}

/// Configuration errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// The config directory could not be determined.
    #[error("Config directory not found")]
    NoDirFound,

    /// Failed to read the config file.
    #[error("Failed to read config file: {0}")]
    ReadFailed(#[source] std::io::Error),

    /// Failed to parse the config file.
    #[error("Failed to parse config: {0}")]
    ParseFailed(#[source] toml::de::Error),

    /// Failed to save the config file.
    #[error("Failed to save config: {0}")]
    SaveFailed(#[source] std::io::Error),

    /// The requested theme was not found.
    #[error("Theme '{0}' not found")]
    ThemeNotFound(String),
}

/// Clipboard errors.
#[derive(Error, Debug, Clone)]
pub enum ClipboardError {
    /// Failed to access the clipboard.
    #[error("Failed to access clipboard: {0}")]
    AccessFailed(String),

    /// Failed to copy content to the clipboard.
    #[error("Failed to copy to clipboard: {0}")]
    CopyFailed(String),
}

// Conversion from ClipboardError to String for backwards compatibility
impl From<ClipboardError> for String {
    fn from(e: ClipboardError) -> Self {
        e.to_string()
    }
}
