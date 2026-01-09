//! Compositor detection logic.

use super::Compositor;
use super::hyprland::HyprlandCompositor;
use super::kwin::KwinCompositor;
use super::niri::NiriCompositor;
use super::noop::NoopCompositor;
use tracing::{info, warn};

/// Detect and create the appropriate compositor client.
///
/// Detection order:
/// 1. Hyprland (via HYPRLAND_INSTANCE_SIGNATURE env var)
/// 2. KDE/KWin (via KDE_SESSION_VERSION env var)
/// 3. Niri     (via NIRI_SOCKET env var)
/// 3. Fallback to NoopCompositor
///
/// The NoopCompositor allows the launcher to function (with applications only)
/// even on unsupported compositors.
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

    // Try Niri
    if let Some(compositor) = NiriCompositor::new() {
        info!("Detected Niri compositor");
        return Box::new(compositor);
    }

    // Fallback to no-op
    warn!("No supported compositor detected, window switching disabled");
    Box::new(NoopCompositor)
}
