//! Daemon initialization functions.
//!
//! Handles setting up logging, IPC, clipboard, compositor, and loading applications.

use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use crate::compositor::{Compositor, detect_compositor};
use crate::config::{ConfigModule, get_combined_modules};
use crate::desktop::cache::load_applications;
use crate::ipc::{IpcServerHandle, client, start_server};
use crate::items::ApplicationItem;

/// Initialize the tracing subscriber for logging.
pub fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    // By default, only log from zlaunch crate at info level
    // Users can override with RUST_LOG environment variable
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("zlaunch=info"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).without_time())
        .with(filter)
        .init();
}

/// Start the IPC server.
///
/// Returns the server handle, or an error if the server couldn't start
/// (e.g., another daemon is already running).
pub fn start_ipc_server(
    event_tx: flume::Sender<crate::app::DaemonEvent>,
) -> Result<IpcServerHandle> {
    match start_server(event_tx) {
        Ok(handle) => Ok(handle),
        Err(e) => {
            if client::is_daemon_running() {
                error!("Daemon already running, exiting");
                std::process::exit(0);
            }
            Err(e)
        }
    }
}

/// Initialize clipboard monitoring if enabled in config.
pub fn init_clipboard_if_enabled() {
    let combined_modules = get_combined_modules();

    if combined_modules.contains(&ConfigModule::Clipboard) {
        // Initialize clipboard history
        crate::clipboard::data::init();
        info!("Initialized clipboard history");

        let _clipboard_monitor_handle = crate::clipboard::monitor::start_monitor();
    }
}

/// Detect and return the compositor.
pub fn init_compositor() -> Arc<dyn Compositor> {
    Arc::from(detect_compositor())
}

/// Apply compositor-specific configuration (e.g., Hyprland blur rules).
pub fn apply_compositor_config() {
    if crate::config::config().hyprland_auto_blur {
        match crate::compositor::hyprland::apply_blur_layer_rules() {
            Ok(true) => info!("Applied Hyprland blur layer rules"),
            Ok(false) => {} // Not on Hyprland, silently skip
            Err(e) => error!("Failed to apply Hyprland blur rules: {}", e),
        }
    }
}

/// Load applications and convert to ApplicationItems.
pub fn load_application_items() -> Vec<ApplicationItem> {
    let entries = load_applications();
    let applications: Vec<ApplicationItem> = entries.into_iter().map(Into::into).collect();
    info!(count = applications.len(), "Loaded applications");
    applications
}
