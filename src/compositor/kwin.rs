//! KDE KWin compositor implementation using D-Bus WindowsRunner API.
//!
//! Uses KWin's krunner interface via D-Bus to enumerate and focus windows.
//! This approach uses the /WindowsRunner D-Bus path which provides direct
//! window listing without needing to capture script print() signals.

use super::{Compositor, WindowInfo};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Command;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedValue;

/// Type alias for KRunner match results from WindowsRunner.Match D-Bus call.
/// Tuple: (match_id, text, subtext, type, relevance, properties)
type KRunnerMatch = (
    String,
    String,
    String,
    i32,
    f64,
    HashMap<String, OwnedValue>,
);

/// KWin compositor client using D-Bus WindowsRunner API.
pub struct KwinCompositor {
    connection: Connection,
}

impl KwinCompositor {
    /// Create a new KWin compositor client.
    ///
    /// Returns None if KDE session is not detected or KWin is not available.
    pub fn new() -> Option<Self> {
        // Check if we're in a KDE session
        if std::env::var("KDE_SESSION_VERSION").is_err() {
            return None;
        }

        // Connect to session D-Bus
        let connection = Connection::session().ok()?;

        // Verify KWin is available by calling supportInformation
        let kwin_proxy = Proxy::new(&connection, "org.kde.KWin", "/KWin", "org.kde.KWin").ok()?;

        let _: String = kwin_proxy.call("supportInformation", &()).ok()?;

        Some(Self { connection })
    }

    /// List windows using the WindowsRunner krunner interface.
    /// Returns tuples of (match_id, title, subtext, type, relevance, properties)
    fn list_windows_via_runner(&self) -> Result<Vec<WindowInfo>> {
        // Create proxy for WindowsRunner
        let runner_proxy = Proxy::new(
            &self.connection,
            "org.kde.KWin",
            "/WindowsRunner",
            "org.kde.krunner1",
        )
        .context("Failed to create WindowsRunner proxy")?;

        // Call Match with empty query to get all windows
        // Returns: a(sssida{sv}) - array of tuples
        let result: Vec<KRunnerMatch> = runner_proxy
            .call("Match", &("",))
            .context("Failed to call WindowsRunner.Match")?;

        let windows: Vec<WindowInfo> = result
            .into_iter()
            .map(
                |(match_id, title, _subtext, _type_id, _relevance, _props)| {
                    // match_id format: "0_{uuid}" - extract the window ID
                    // The "0_" prefix indicates action index (0 = activate)
                    let window_id = match_id
                        .strip_prefix("0_")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| match_id.clone());

                    // Try to extract app class from the title (often "Title - AppName")
                    // This is a heuristic - the actual class isn't directly available
                    let class = title.rsplit(" - ").next().unwrap_or(&title).to_string();

                    WindowInfo {
                        address: window_id,
                        title: title.clone(),
                        class,
                        workspace: 1,   // WindowsRunner doesn't expose workspace info
                        focused: false, // We can't easily determine this from krunner
                    }
                },
            )
            .collect();

        Ok(windows)
    }

    /// Focus a window using the WindowsRunner Run method.
    fn focus_window_via_runner(&self, window_id: &str) -> Result<()> {
        let runner_proxy = Proxy::new(
            &self.connection,
            "org.kde.KWin",
            "/WindowsRunner",
            "org.kde.krunner1",
        )
        .context("Failed to create WindowsRunner proxy")?;

        // match_id needs the "0_" prefix for the activate action
        let match_id = format!("0_{}", window_id);

        // Run with empty action_id (default action = activate)
        let _: () = runner_proxy
            .call("Run", &(&match_id, ""))
            .context("Failed to call WindowsRunner.Run")?;

        Ok(())
    }
}

impl Compositor for KwinCompositor {
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        self.list_windows_via_runner()
    }

    fn focus_window(&self, window_id: &str) -> Result<()> {
        // First try the krunner approach
        if let Ok(()) = self.focus_window_via_runner(window_id) {
            return Ok(());
        }

        // Fallback: use qdbus to activate window
        let status = Command::new("qdbus")
            .args([
                "org.kde.KWin",
                "/WindowsRunner",
                "org.kde.krunner1.Run",
                &format!("0_{}", window_id),
                "",
            ])
            .status()
            .context("Failed to run qdbus")?;

        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("qdbus command failed with status: {}", status)
        }
    }

    fn name(&self) -> &'static str {
        "KWin"
    }
}
