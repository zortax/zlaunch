//! Daemon reload functionality.
//!
//! Handles signaling and executing a daemon reload (restart).

use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

/// Flag to signal that the daemon should reload (exec) after GPUI exits.
static RELOAD_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Set whether a reload is requested.
pub fn set_reload_requested(value: bool) {
    RELOAD_REQUESTED.store(value, Ordering::SeqCst);
}

/// Check if a reload is requested.
pub fn is_reload_requested() -> bool {
    RELOAD_REQUESTED.load(Ordering::SeqCst)
}

/// Execute a reload by replacing the current process with a fresh daemon.
///
/// This function uses `exec()` to replace the current process image,
/// so it never returns on success.
pub fn exec_reload() -> Result<()> {
    use std::os::unix::process::CommandExt;

    info!("Executing daemon reload...");

    // Get the path to the current executable
    let exe = std::env::current_exe().context("Failed to get current executable path")?;

    // Ensure socket is removed (IpcServerHandle should have dropped, but be safe)
    let socket_path = crate::ipc::get_socket_path();
    if socket_path.exists()
        && let Err(e) = std::fs::remove_file(&socket_path)
    {
        tracing::warn!("Failed to clean up socket file: {}", e);
    }

    // exec() replaces the current process - this never returns on success
    // zlaunch daemon starts with no arguments
    let err = std::process::Command::new(&exe).exec();

    // If we get here, exec failed
    Err(anyhow::anyhow!("Failed to exec: {}", err))
}
