//! Hyprland compositor implementation using IPC socket.

use super::{Compositor, WindowInfo};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Hyprland compositor client using IPC socket communication.
pub struct HyprlandCompositor {
    socket_path: PathBuf,
}

impl HyprlandCompositor {
    /// Create a new Hyprland compositor client.
    ///
    /// Returns None if the required environment variables are not set.
    pub fn new() -> Option<Self> {
        let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());

        let socket_path = PathBuf::from(format!("{}/hypr/{}/.socket.sock", runtime_dir, signature));

        Some(Self { socket_path })
    }

    /// Send a command to Hyprland and receive the response.
    fn send_command(&self, cmd: &str) -> Result<String> {
        let mut stream = UnixStream::connect(&self.socket_path).with_context(|| {
            format!(
                "Failed to connect to Hyprland socket: {:?}",
                self.socket_path
            )
        })?;

        stream
            .write_all(cmd.as_bytes())
            .context("Failed to write command to Hyprland socket")?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .context("Failed to read response from Hyprland socket")?;

        Ok(response)
    }
}

impl Compositor for HyprlandCompositor {
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        // j/clients returns JSON output
        let json = self.send_command("j/clients")?;
        let clients: Vec<HyprlandClient> =
            serde_json::from_str(&json).context("Failed to parse Hyprland clients JSON")?;

        let windows = clients
            .into_iter()
            // Filter out special windows
            .filter(|c| {
                // Exclude unmapped or hidden windows
                if !c.mapped || c.hidden {
                    return false;
                }
                // Exclude zlaunch itself
                if c.class.to_lowercase() == "zlaunch" {
                    return false;
                }
                // Exclude windows with empty class (usually special windows)
                if c.class.is_empty() {
                    return false;
                }
                true
            })
            .map(|c| {
                let focused = c.is_focused();
                let workspace = c.workspace.id;
                WindowInfo {
                    address: c.address,
                    title: if c.title.is_empty() {
                        c.class.clone()
                    } else {
                        c.title
                    },
                    class: c.class,
                    workspace,
                    focused,
                }
            })
            .collect();

        Ok(windows)
    }

    fn focus_window(&self, window_id: &str) -> Result<()> {
        let cmd = format!("dispatch focuswindow address:{}", window_id);
        self.send_command(&cmd)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "Hyprland"
    }
}

/// Hyprland client (window) information from IPC.
#[derive(Debug, Deserialize)]
struct HyprlandClient {
    address: String,
    title: String,
    class: String,
    workspace: HyprlandWorkspace,
    /// 0 means currently focused, higher numbers mean less recently focused
    #[serde(rename = "focusHistoryID")]
    focus_history_id: i32,
    #[serde(default)]
    mapped: bool,
    #[serde(default)]
    hidden: bool,
}

impl HyprlandClient {
    /// Check if this window is currently focused (focusHistoryID == 0)
    fn is_focused(&self) -> bool {
        self.focus_history_id == 0
    }
}

/// Hyprland workspace information.
#[derive(Debug, Deserialize)]
struct HyprlandWorkspace {
    id: i32,
}

/// Apply blur layer rules for zlaunch on Hyprland.
///
/// This sets up transparency and blur effects via Hyprland IPC.
/// Returns `Ok(true)` if rules were applied, `Ok(false)` if not on Hyprland.
pub fn apply_blur_layer_rules() -> Result<bool> {
    // Check if we're on Hyprland
    let Some(compositor) = HyprlandCompositor::new() else {
        return Ok(false);
    };

    let rules = [
        "blur,zlaunch",
        "ignorezero,zlaunch",
        "blurpopups,zlaunch",
        "ignorealpha 0.35,zlaunch",
    ];

    for rule in rules {
        let cmd = format!("keyword layerrule {}", rule);
        compositor.send_command(&cmd)?;
    }

    Ok(true)
}
