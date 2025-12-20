use anyhow::Result;
use gpui::{Application, QuitMode, hsla};
use gpui_component::theme::{Theme, ThemeMode};
use std::sync::Arc;
use tracing::{error, info};

use crate::app::window::LauncherWindow;
use crate::app::{DaemonEvent, WindowEvent, create_daemon_channel, window};
use crate::assets::CombinedAssets;
use crate::compositor::{Compositor, detect_compositor};
use crate::config::{ConfigModule, config};
use crate::desktop::cache::load_applications;
use crate::desktop::capture_session_environment;
use crate::ipc::client;
use crate::ipc::{IpcServerHandle, start_server};
use crate::items::ApplicationItem;
use crate::ui::init_launcher;

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

/// Run the launcher daemon.
/// This is the main entry point when no subcommand is provided.
pub fn run() -> Result<()> {
    init_logging();
    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting zlaunch daemon"
    );

    // Create unified event channel
    let (event_tx, event_rx) = create_daemon_channel();

    // Start tarpc IPC server
    let _ipc_handle: IpcServerHandle = match start_server(event_tx.clone()) {
        Ok(handle) => handle,
        Err(e) => {
            if client::is_daemon_running() {
                error!("Daemon already running, exiting");
                return Ok(());
            }
            return Err(e);
        }
    };

    // Initialize config from file (single source of truth)
    crate::config::init_config();

    // Capture the full session environment early, including from systemd user session.
    // This ensures launched applications get proper theming variables.
    capture_session_environment();

    // Get the config disabled modules
    let disabled_modules = config().disabled_modules.unwrap_or_default();

    // Start clipboard monitor
    if !disabled_modules.contains(&ConfigModule::Clipboard) {
        // Initialize clipboard history
        crate::clipboard::data::init();
        info!("Initialized clipboard history");

        let _clipboard_monitor_handle = crate::clipboard::monitor::start_monitor();
    }

    // Detect compositor for window switching support
    let compositor: Arc<dyn Compositor> = Arc::from(detect_compositor());

    // Apply Hyprland blur layer rules if enabled
    if crate::config::config().hyprland_auto_blur {
        match crate::compositor::hyprland::apply_blur_layer_rules() {
            Ok(true) => info!("Applied Hyprland blur layer rules"),
            Ok(false) => {} // Not on Hyprland, silently skip
            Err(e) => error!("Failed to apply Hyprland blur rules: {}", e),
        }
    }

    // Load applications and convert to ApplicationItems
    let entries = load_applications();
    let applications: Vec<ApplicationItem> = entries.into_iter().map(Into::into).collect();
    info!(count = applications.len(), "Loaded applications");

    Application::new()
        .with_assets(CombinedAssets)
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx| {
            gpui_component::init(cx);
            init_launcher(cx);
            Theme::change(ThemeMode::Dark, None, cx);

            // Customize theme for transparent background and no borders
            configure_theme(cx);

            let applications_clone = applications.clone();
            let compositor_clone = compositor.clone();
            let mut launcher_window: Option<LauncherWindow> = None;
            let mut visible = false;

            // Main event loop - async wait on channel, no polling needed
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                while let Ok(event) = event_rx.recv_async().await {
                    match event {
                        DaemonEvent::Window(WindowEvent::RequestHide) if visible => {
                            let _ = cx.update(|cx| {
                                if let Some(ref lw) = launcher_window {
                                    window::close_window(&lw.handle, cx);
                                }
                            });
                            launcher_window = None;
                            visible = false;
                        }

                        DaemonEvent::Show { response_tx } => {
                            let result = if !visible {
                                cx.update(|cx| {
                                    match window::create_and_show_window(
                                        applications_clone.clone(),
                                        compositor_clone.clone(),
                                        event_tx.clone(),
                                        cx,
                                    ) {
                                        Ok(lw) => {
                                            launcher_window = Some(lw);
                                            visible = true;
                                            Ok(())
                                        }
                                        Err(e) => {
                                            error!(%e, "Failed to create window");
                                            Err(format!("Failed to create window: {}", e))
                                        }
                                    }
                                })
                                .unwrap_or(Err("Failed to update app".to_string()))
                            } else {
                                Ok(()) // Already visible
                            };
                            let _ = response_tx.send(result);
                        }

                        DaemonEvent::Hide { response_tx } => {
                            if visible {
                                let _ = cx.update(|cx| {
                                    if let Some(ref lw) = launcher_window {
                                        window::close_window(&lw.handle, cx);
                                    }
                                });
                                launcher_window = None;
                                visible = false;
                            }
                            let _ = response_tx.send(Ok(()));
                        }

                        DaemonEvent::Toggle { response_tx } => {
                            let result = if visible {
                                let _ = cx.update(|cx| {
                                    if let Some(ref lw) = launcher_window {
                                        window::close_window(&lw.handle, cx);
                                    }
                                });
                                launcher_window = None;
                                visible = false;
                                Ok(())
                            } else {
                                cx.update(|cx| {
                                    match window::create_and_show_window(
                                        applications_clone.clone(),
                                        compositor_clone.clone(),
                                        event_tx.clone(),
                                        cx,
                                    ) {
                                        Ok(lw) => {
                                            launcher_window = Some(lw);
                                            visible = true;
                                            Ok(())
                                        }
                                        Err(e) => {
                                            error!(%e, "Failed to create window");
                                            Err(format!("Failed to create window: {}", e))
                                        }
                                    }
                                })
                                .unwrap_or(Err("Failed to update app".to_string()))
                            };
                            let _ = response_tx.send(result);
                        }

                        DaemonEvent::Quit { response_tx } => {
                            let _ = response_tx.send(Ok(()));
                            let _ = cx.update(|cx| {
                                cx.quit();
                            });
                        }

                        DaemonEvent::SetTheme { name, response_tx } => {
                            let result = handle_set_theme(&name);
                            // If window is open, refresh the theme on the view
                            if visible && let Some(ref lw) = launcher_window {
                                let view = lw.launcher_view.clone();
                                let _ = cx.update(|cx| {
                                    view.update(cx, |launcher, cx| {
                                        launcher.refresh_theme(cx);
                                    });
                                });
                            }
                            let _ = response_tx.send(result);
                        }

                        _ => {}
                    }
                }
            })
            .detach();
        });

    Ok(())
}

/// Handle the SetTheme IPC command.
fn handle_set_theme(name: &str) -> Result<(), String> {
    // Validate theme exists before updating config
    crate::config::load_theme(name).ok_or_else(|| format!("Theme '{}' not found", name))?;

    // Update config (persists to disk if config file exists)
    crate::config::update_config(|config| {
        config.theme = name.to_string();
    });

    // Sync the theme cache from the updated config
    crate::ui::theme::sync_theme_from_config();

    Ok(())
}

/// Configure the global theme for transparent launcher appearance.
fn configure_theme(cx: &mut gpui::App) {
    let theme = Theme::global_mut(cx);
    theme.background = hsla(0.0, 0.0, 0.0, 0.0); // Fully transparent
    theme.window_border = hsla(0.0, 0.0, 0.0, 0.0); // No window border
    theme.border = hsla(0.0, 0.0, 1.0, 0.1); // Subtle separator between search and list
    theme.list_active_border = hsla(0.0, 0.0, 0.0, 0.0); // No selection border
    theme.list_active = hsla(0.0, 0.0, 0.0, 0.0); // Fully transparent - we handle selection ourselves
    theme.list_hover = hsla(0.0, 0.0, 0.0, 0.0); // Fully transparent - we handle hover ourselves
    theme.mono_font_family = "Mononoki Nerd Font Mono".into(); // Monospace font for code blocks
}
