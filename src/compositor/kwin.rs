//! KDE KWin compositor implementation using DBus.

use super::{Compositor, WindowInfo};
use anyhow::{Context, Result, anyhow};
use std::process::Command;

/// KWin compositor client using DBus communication.
pub struct KwinCompositor {
    _private: (),
}

impl KwinCompositor {
    /// Create a new KWin compositor client.
    ///
    /// Returns None if KDE session is not detected.
    pub fn new() -> Option<Self> {
        // Check if we're in a KDE session
        if std::env::var("KDE_SESSION_VERSION").is_err() {
            return None;
        }

        // Verify KWin is available by checking the DBus service
        let output = Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                "--dest=org.kde.KWin",
                "/KWin",
                "org.kde.KWin.supportInformation",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        Some(Self { _private: () })
    }

    /// Get window list using KWin scripting interface.
    fn get_windows_via_script(&self) -> Result<Vec<WindowInfo>> {
        // TODO: Implement native KWin scripting API for better integration
        // For now, use wmctrl as it's reliable and works on KDE
        self.get_windows_via_wmctrl()
    }

    /// Fallback: get windows using wmctrl command.
    fn get_windows_via_wmctrl(&self) -> Result<Vec<WindowInfo>> {
        let output = Command::new("wmctrl").args(["-l", "-p"]).output().context(
            "Failed to run wmctrl. Please install wmctrl for KDE window switching support.",
        )?;

        if !output.status.success() {
            return Err(anyhow!(
                "wmctrl failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut windows = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let window_id = parts[0].to_string();
                let workspace: i32 = parts[1].parse().unwrap_or(0);
                // Skip desktop and special windows (workspace -1)
                if workspace < 0 {
                    continue;
                }
                // parts[2] is PID, parts[3] is hostname
                // Everything after is the title
                let title = parts[4..].join(" ");

                // Skip zlaunch itself
                if title.to_lowercase().contains("zlaunch") {
                    continue;
                }

                windows.push(WindowInfo {
                    address: window_id,
                    title,
                    class: String::new(),     // wmctrl -l doesn't give class
                    workspace: workspace + 1, // wmctrl uses 0-indexed
                    focused: false,           // wmctrl doesn't tell us this
                });
            }
        }

        // Try to get window classes with wmctrl -lx
        if let Ok(output) = Command::new("wmctrl").args(["-lx"]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let window_id = parts[0];
                        let class_parts: Vec<&str> = parts[2].split('.').collect();
                        let class = class_parts.last().unwrap_or(&"").to_string();

                        // Update the window with its class
                        if let Some(win) = windows.iter_mut().find(|w| w.address == window_id) {
                            win.class = class;
                        }
                    }
                }
            }
        }

        Ok(windows)
    }

    /// Focus window using wmctrl.
    fn focus_via_wmctrl(&self, window_id: &str) -> Result<()> {
        let output = Command::new("wmctrl")
            .args(["-i", "-a", window_id])
            .output()
            .context("Failed to run wmctrl")?;

        if !output.status.success() {
            return Err(anyhow!(
                "wmctrl focus failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}

impl Compositor for KwinCompositor {
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        self.get_windows_via_script()
    }

    fn focus_window(&self, window_id: &str) -> Result<()> {
        self.focus_via_wmctrl(window_id)
    }

    fn name(&self) -> &'static str {
        "KWin"
    }
}
