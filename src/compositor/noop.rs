//! No-op compositor implementation for unsupported environments.

use super::{Compositor, WindowInfo};

/// A no-op compositor that returns empty results.
///
/// Used as a fallback when no supported compositor is detected.
/// This allows the launcher to function (with applications only)
/// even on unsupported compositors.
pub struct NoopCompositor;

impl Compositor for NoopCompositor {
    fn list_windows(&self) -> anyhow::Result<Vec<WindowInfo>> {
        Ok(Vec::new())
    }

    fn focus_window(&self, _window_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "Noop"
    }
}
