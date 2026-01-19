//! Event handling for the daemon.
//!
//! Processes DaemonEvent messages from IPC and manages window state.

use std::sync::Arc;
use tracing::debug;

use crate::app::window::LauncherWindow;
use crate::app::{DaemonEvent, WindowEvent, window};
use crate::compositor::Compositor;
use crate::config::get_default_modes;
use crate::error::IpcError;
use crate::items::ApplicationItem;

use super::reload::set_reload_requested;
use super::theme::handle_set_theme;

/// Window state manager for the daemon.
pub struct WindowState {
    /// The current launcher window, if open.
    pub launcher_window: Option<LauncherWindow>,
    /// Whether the window is visible.
    pub visible: bool,
}

impl WindowState {
    /// Create a new window state.
    pub fn new() -> Self {
        Self {
            launcher_window: None,
            visible: false,
        }
    }

    /// Close the window if it exists.
    pub fn close(&mut self, cx: &mut gpui::App) {
        if let Some(ref lw) = self.launcher_window {
            window::close_window(&lw.handle, cx);
        }
        self.launcher_window = None;
        self.visible = false;
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the main event loop, processing DaemonEvents.
///
/// This function is spawned as an async task within the GPUI application.
pub async fn run_event_loop(
    event_rx: flume::Receiver<DaemonEvent>,
    event_tx: flume::Sender<DaemonEvent>,
    applications: Vec<ApplicationItem>,
    compositor: Arc<dyn Compositor>,
    cx: &mut gpui::AsyncApp,
) {
    let mut window_state = WindowState::new();

    while let Ok(event) = event_rx.recv_async().await {
        match event {
            DaemonEvent::Window(WindowEvent::RequestHide) if window_state.visible => {
                let _ = cx.update(|cx| {
                    window_state.close(cx);
                });
            }

            DaemonEvent::Show { modes, response_tx } => {
                let result = handle_show(
                    &mut window_state,
                    modes,
                    &applications,
                    &compositor,
                    &event_tx,
                    cx,
                );
                if response_tx.send(result).is_err() {
                    debug!("Client disconnected before receiving response");
                }
            }

            DaemonEvent::Hide { response_tx } => {
                if window_state.visible {
                    let _ = cx.update(|cx| {
                        window_state.close(cx);
                    });
                }
                if response_tx.send(Ok(())).is_err() {
                    debug!("Client disconnected before receiving response");
                }
            }

            DaemonEvent::Toggle { modes, response_tx } => {
                debug!("Processing Toggle event, visible={}", window_state.visible);
                let result = if window_state.visible {
                    let _ = cx.update(|cx| {
                        window_state.close(cx);
                    });
                    Ok(())
                } else {
                    handle_show(
                        &mut window_state,
                        modes,
                        &applications,
                        &compositor,
                        &event_tx,
                        cx,
                    )
                };
                if response_tx.send(result).is_err() {
                    debug!("Client disconnected before receiving response");
                }
            }

            DaemonEvent::Quit { response_tx } => {
                if response_tx.send(Ok(())).is_err() {
                    debug!("Client disconnected before receiving quit response");
                }
                let _ = cx.update(|cx| {
                    cx.quit();
                });
                return;
            }

            DaemonEvent::SetTheme { name, response_tx } => {
                let result = handle_set_theme(&name);
                // If window is open, refresh the theme on the view
                if window_state.visible
                    && let Some(ref lw) = window_state.launcher_window
                {
                    let view = lw.launcher_view.clone();
                    let _ = cx.update(|cx| {
                        view.update(cx, |launcher, cx| {
                            launcher.refresh_theme(cx);
                        });
                    });
                }
                if response_tx.send(result).is_err() {
                    debug!("Client disconnected before receiving theme response");
                }
            }

            DaemonEvent::Reload { response_tx } => {
                // Send response FIRST so client sees success before we exit
                if response_tx.send(Ok(())).is_err() {
                    debug!("Client disconnected before receiving reload response");
                }

                // Close window if visible
                if window_state.visible {
                    let _ = cx.update(|cx| {
                        window_state.close(cx);
                    });
                }

                // Signal reload and quit GPUI
                set_reload_requested(true);
                let _ = cx.update(|cx| {
                    cx.quit();
                });
                return;
            }

            _ => {}
        }
    }
}

/// Handle the Show event - create and show the launcher window.
fn handle_show(
    window_state: &mut WindowState,
    modes: Option<Vec<crate::config::LauncherMode>>,
    applications: &[ApplicationItem],
    compositor: &Arc<dyn Compositor>,
    event_tx: &flume::Sender<DaemonEvent>,
    cx: &mut gpui::AsyncApp,
) -> Result<(), IpcError> {
    if window_state.visible {
        return Ok(()); // Already visible
    }

    // Use provided modes or fall back to configured defaults
    let effective_modes = modes.unwrap_or_else(get_default_modes);

    cx.update(|cx| {
        match window::create_and_show_window(
            applications.to_vec(),
            compositor.clone(),
            effective_modes,
            event_tx.clone(),
            cx,
        ) {
            Ok(lw) => {
                window_state.launcher_window = Some(lw);
                window_state.visible = true;
                Ok(())
            }
            Err(e) => {
                tracing::error!(%e, "Failed to create window");
                Err(IpcError::Internal(format!(
                    "Failed to create window: {}",
                    e
                )))
            }
        }
    })
    .unwrap_or(Err(IpcError::Internal("Failed to update app".into())))
}
