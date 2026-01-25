//! tarpc server implementation for the IPC daemon.

use crate::app::DaemonEvent;
use crate::config::LauncherMode;
use crate::error::IpcError;
use crate::ipc::commands::{ThemeInfo, ZlaunchService};
use crate::items::ThemeSource;
use futures::prelude::*;
use std::path::PathBuf;
use tarpc::context::Context;
use tarpc::server::{BaseChannel, Channel};
use tarpc::tokio_serde::formats::Json;
use tokio::net::UnixListener;
use tokio::sync::oneshot;
use tokio_util::codec::LengthDelimitedCodec;

/// Handle for the IPC server, cleans up socket on drop.
pub struct IpcServerHandle {
    socket_path: PathBuf,
}

impl Drop for IpcServerHandle {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.socket_path) {
            // Don't warn if the file doesn't exist - that's expected if the socket
            // was never created or was already cleaned up
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("Failed to clean up IPC socket: {}", e);
            }
        }
    }
}

/// Get the socket path for the IPC server.
pub fn get_socket_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join("zlaunch.sock")
}

/// Check if another daemon instance is running.
pub fn is_daemon_running() -> bool {
    let socket_path = get_socket_path();
    std::os::unix::net::UnixStream::connect(&socket_path).is_ok()
}

/// The tarpc server implementation.
#[derive(Clone)]
struct ZlaunchServer {
    event_tx: flume::Sender<DaemonEvent>,
}

impl ZlaunchService for ZlaunchServer {
    async fn show(self, _: Context, modes: Option<Vec<LauncherMode>>) -> Result<(), IpcError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.event_tx
            .send(DaemonEvent::Show { modes, response_tx })
            .map_err(|_| IpcError::ChannelClosed)?;
        response_rx.await.unwrap_or(Err(IpcError::ResponseClosed))
    }

    async fn hide(self, _: Context) -> Result<(), IpcError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.event_tx
            .send(DaemonEvent::Hide { response_tx })
            .map_err(|_| IpcError::ChannelClosed)?;
        response_rx.await.unwrap_or(Err(IpcError::ResponseClosed))
    }

    async fn toggle(self, _: Context, modes: Option<Vec<LauncherMode>>) -> Result<(), IpcError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.event_tx
            .send(DaemonEvent::Toggle { modes, response_tx })
            .map_err(|_| IpcError::ChannelClosed)?;
        response_rx.await.unwrap_or(Err(IpcError::ResponseClosed))
    }

    async fn quit(self, _: Context) -> Result<(), IpcError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.event_tx
            .send(DaemonEvent::Quit { response_tx })
            .map_err(|_| IpcError::ChannelClosed)?;
        response_rx.await.unwrap_or(Err(IpcError::ResponseClosed))
    }

    async fn reload(self, _: Context) -> Result<(), IpcError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.event_tx
            .send(DaemonEvent::Reload { response_tx })
            .map_err(|_| IpcError::ChannelClosed)?;
        response_rx.await.unwrap_or(Err(IpcError::ResponseClosed))
    }

    async fn list_themes(self, _: Context) -> Vec<ThemeInfo> {
        // Read-only operation - can be answered directly
        crate::config::list_all_themes_with_source()
            .into_iter()
            .map(|(name, source)| ThemeInfo {
                name,
                is_bundled: matches!(source, ThemeSource::Bundled),
            })
            .collect()
    }

    async fn get_current_theme(self, _: Context) -> String {
        // Read-only operation - can be answered directly
        crate::ui::theme::theme().name
    }

    async fn set_theme(self, _: Context, name: String) -> Result<(), IpcError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.event_tx
            .send(DaemonEvent::SetTheme { name, response_tx })
            .map_err(|_| IpcError::ChannelClosed)?;
        response_rx.await.unwrap_or(Err(IpcError::ResponseClosed))
    }
}

/// Prepare the IPC socket, checking for existing instances.
///
/// This should be called early, before the GPUI application starts.
/// Returns Ok with the socket path on success, Err if another instance is running.
pub fn prepare_socket() -> anyhow::Result<PathBuf> {
    let socket_path = get_socket_path();

    // Check for existing instance
    if socket_path.exists() {
        if is_daemon_running() {
            anyhow::bail!("Another instance is already running");
        }
        // Remove stale socket
        std::fs::remove_file(&socket_path)?;
    }

    Ok(socket_path)
}

/// Start the tarpc IPC server on the shared tokio runtime.
///
/// This should be called inside the GPUI run closure, after the tokio runtime is initialized.
pub fn start_server(event_tx: flume::Sender<DaemonEvent>, cx: &gpui::App) -> IpcServerHandle {
    let socket_path = get_socket_path();
    let socket_path_clone = socket_path.clone();

    // Spawn the server on the shared tokio runtime
    crate::tokio_runtime::spawn(cx, async move {
        let listener = UnixListener::bind(&socket_path_clone).expect("Failed to bind IPC socket");

        tracing::info!("IPC server listening on {:?}", socket_path_clone);

        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::warn!("Failed to accept IPC connection: {}", e);
                    continue;
                }
            };

            let framed = tokio_util::codec::Framed::new(stream, LengthDelimitedCodec::new());
            let transport = tarpc::serde_transport::new(framed, Json::default());

            let server = ZlaunchServer {
                event_tx: event_tx.clone(),
            };

            let channel = BaseChannel::with_defaults(transport);

            tokio::spawn(
                channel
                    .execute(server.serve())
                    .for_each(|response| async move {
                        tokio::spawn(response);
                    }),
            );
        }
    });

    IpcServerHandle { socket_path }
}
