//! tarpc service definition for IPC communication.

use serde::{Deserialize, Serialize};

/// Theme information returned by the IPC service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    /// Theme name
    pub name: String,
    /// Whether this is a bundled theme (vs user-defined)
    pub is_bundled: bool,
}

/// The zlaunch RPC service definition.
#[tarpc::service]
pub trait ZlaunchService {
    /// Show the launcher window.
    async fn show() -> Result<(), String>;

    /// Hide the launcher window.
    async fn hide() -> Result<(), String>;

    /// Toggle the launcher window visibility.
    async fn toggle() -> Result<(), String>;

    /// Quit the daemon.
    async fn quit() -> Result<(), String>;

    /// Reload the daemon (fully restart the process).
    async fn reload() -> Result<(), String>;

    /// List all available themes.
    async fn list_themes() -> Vec<ThemeInfo>;

    /// Get the current theme name.
    async fn get_current_theme() -> String;

    /// Set the active theme by name.
    /// Returns Ok(()) if successful, Err with message if theme not found.
    async fn set_theme(name: String) -> Result<(), String>;
}
