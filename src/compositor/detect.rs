//! Compositor detection logic.

use super::Compositor;
use super::noop::NoopCompositor;

#[cfg(unix)]
use super::hyprland::HyprlandCompositor;
#[cfg(unix)]
use super::kwin::KwinCompositor;

use tracing::{info, warn};

/// Detect and create the appropriate compositor client.
///
/// Detection order (Unix only):
/// 1. Hyprland (via HYPRLAND_INSTANCE_SIGNATURE env var)
/// 2. KDE/KWin (via KDE_SESSION_VERSION env var)
/// 3. Fallback to NoopCompositor
///
/// On Windows, always returns NoopCompositor as window switching
/// is not yet supported.
///
/// The NoopCompositor allows the launcher to function (with applications only)
/// even on unsupported compositors.
#[cfg(unix)]
pub fn detect_compositor() -> Box<dyn Compositor> {
    // Try Hyprland first
    if let Some(compositor) = HyprlandCompositor::new() {
        info!("Detected Hyprland compositor");
        return Box::new(compositor);
    }

    // Try KWin
    if let Some(compositor) = KwinCompositor::new() {
        info!("Detected KWin compositor");
        return Box::new(compositor);
    }

    // Fallback to no-op
    warn!("No supported compositor detected, window switching disabled");
    Box::new(NoopCompositor)
}

#[cfg(windows)]
pub fn detect_compositor() -> Box<dyn Compositor> {
    // Window switching is not yet supported on Windows
    info!("Window switching not supported on Windows");
    Box::new(NoopCompositor)
}
