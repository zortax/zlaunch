//! Compositor abstraction for window management.
//!
//! This module provides a trait-based abstraction for interacting with
//! Wayland compositors to list windows and switch focus. Implementations
//! are provided for Hyprland (IPC socket) and KDE/KWin (DBus).

mod detect;
pub mod hyprland;
mod kwin;
mod noop;

pub use detect::detect_compositor;

use std::fmt;

/// Information about an open window from the compositor.
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Unique window identifier (compositor-specific, e.g., "0x5678abcd" for Hyprland)
    pub address: String,
    /// Window title
    pub title: String,
    /// Application class/ID (e.g., "firefox", "org.kde.dolphin")
    pub class: String,
    /// Workspace number
    pub workspace: i32,
    /// Whether this window is currently focused
    pub focused: bool,
}

/// Trait for compositor window management operations.
///
/// Implementations must be thread-safe (Send + Sync) as the compositor
/// may be accessed from different threads in the daemon.
pub trait Compositor: Send + Sync {
    /// List all open windows.
    ///
    /// Returns only "normal" user windows - layer shell windows (panels, bars),
    /// the launcher itself, and other special windows should be filtered out.
    fn list_windows(&self) -> anyhow::Result<Vec<WindowInfo>>;

    /// Focus/activate a window by its address.
    ///
    /// The address format is compositor-specific.
    fn focus_window(&self, window_id: &str) -> anyhow::Result<()>;

    /// Get the compositor name for logging/debugging.
    fn name(&self) -> &'static str;
}

impl fmt::Debug for dyn Compositor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Compositor({})", self.name())
    }
}
