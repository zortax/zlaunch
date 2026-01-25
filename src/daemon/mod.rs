//! Daemon module for zlaunch.
//!
//! The daemon is the main process that stays running, handling IPC commands
//! and managing the launcher window lifecycle.

mod event_handler;
mod init;
mod reload;
mod theme;
mod watcher;

use anyhow::Result;
use gpui::{Application, QuitMode};
use gpui_component::theme::{Theme, ThemeMode};
use tracing::info;

use crate::app::create_daemon_channel;
use crate::assets::CombinedAssets;
use crate::ui::init_launcher;

pub use init::init_logging;

/// Run the launcher daemon.
///
/// This is the main entry point when no subcommand is provided.
/// It initializes services, starts the GPUI application, and runs the event loop.
pub fn run() -> Result<()> {
    init::init_logging();
    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting zlaunch daemon"
    );

    // Create unified event channel
    let (event_tx, event_rx) = create_daemon_channel();

    // Prepare IPC socket (check for existing instance)
    init::prepare_ipc_socket()?;

    // Initialize config from file (single source of truth)
    crate::config::init_config();

    // Capture the full session environment early
    crate::desktop::capture_session_environment();

    // Start clipboard monitor if enabled
    init::init_clipboard_if_enabled();

    // Detect compositor for window switching support
    let compositor = init::init_compositor();

    // Apply compositor-specific configuration
    init::apply_compositor_config();

    // Load applications
    let applications = init::load_application_items();

    // Run GPUI application
    Application::new()
        .with_assets(CombinedAssets)
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx| {
            gpui_component::init(cx);
            init_launcher(cx);
            Theme::change(ThemeMode::Dark, None, cx);

            // Initialize shared tokio runtime
            crate::tokio_runtime::init(cx);

            // Start IPC server on shared tokio runtime
            let _ipc_handle = init::start_ipc_server(event_tx.clone(), cx);

            // Configure theme for transparent background
            theme::configure_theme(cx);

            // Clone for move into async block
            let applications = applications.clone();
            let compositor = compositor.clone();
            let event_tx_clone = event_tx.clone();

            // Spawn file watcher on shared tokio runtime
            let event_tx_for_watcher = event_tx.clone();
            crate::tokio_runtime::spawn(cx, watcher::run_watcher_loop(event_tx_for_watcher));

            // Main event loop (runs on GPUI executor)
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                event_handler::run_event_loop(
                    event_rx,
                    event_tx_clone,
                    applications,
                    compositor,
                    cx,
                )
                .await;
            })
            .detach();
        });

    // After GPUI exits, check if we should reload
    if reload::is_reload_requested() {
        reload::exec_reload()?;
    }

    Ok(())
}
