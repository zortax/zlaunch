use crate::ipc::commands::{Command, Response};
use std::io::{Read, Write};
use std::sync::Arc;

// Platform-specific imports and types
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::path::PathBuf;

#[cfg(windows)]
use std::net::{TcpListener, TcpStream};

/// Platform-specific listener type alias
#[cfg(unix)]
pub type PlatformListener = UnixListener;
#[cfg(windows)]
pub type PlatformListener = TcpListener;

/// IPC server that listens for commands from external clients.
pub struct IpcServer {
    listener: Arc<PlatformListener>,
    #[cfg(unix)]
    socket_path: PathBuf,
}

impl IpcServer {
    #[cfg(unix)]
    pub fn new() -> anyhow::Result<Self> {
        let socket_path = get_socket_path();

        if socket_path.exists() {
            if UnixStream::connect(&socket_path).is_ok() {
                anyhow::bail!("Another instance is already running");
            }
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        // Keep in blocking mode for accept_blocking()

        Ok(Self {
            listener: Arc::new(listener),
            socket_path,
        })
    }

    #[cfg(windows)]
    pub fn new() -> anyhow::Result<Self> {
        // Use TCP on localhost for Windows IPC
        // Try to bind to the port, if it fails another instance might be running
        let listener = match TcpListener::bind(get_socket_addr()) {
            Ok(l) => l,
            Err(e) => {
                // Check if another instance is running
                if TcpStream::connect(get_socket_addr()).is_ok() {
                    anyhow::bail!("Another instance is already running");
                }
                return Err(e.into());
            }
        };

        Ok(Self {
            listener: Arc::new(listener),
        })
    }

    /// Get a clone of the listener Arc for use in background threads.
    pub fn listener(&self) -> Arc<PlatformListener> {
        Arc::clone(&self.listener)
    }

    /// Blocking accept - waits for a connection and returns the command.
    /// This should be called from a background thread.
    #[cfg(unix)]
    pub fn accept_blocking(listener: &UnixListener) -> Option<Command> {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0u8; 1024];
                let n = stream.read(&mut buf).ok()?;
                let cmd: Command = serde_json::from_slice(&buf[..n]).ok()?;

                let response = Response::Ok;
                let response_bytes = serde_json::to_vec(&response).ok()?;
                let _ = stream.write_all(&response_bytes);

                Some(cmd)
            }
            Err(_) => None,
        }
    }

    #[cfg(windows)]
    pub fn accept_blocking(listener: &TcpListener) -> Option<Command> {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0u8; 1024];
                let n = stream.read(&mut buf).ok()?;
                let cmd: Command = serde_json::from_slice(&buf[..n]).ok()?;

                let response = Response::Ok;
                let response_bytes = serde_json::to_vec(&response).ok()?;
                let _ = stream.write_all(&response_bytes);

                Some(cmd)
            }
            Err(_) => None,
        }
    }
}

#[cfg(unix)]
impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

#[cfg(unix)]
pub fn get_socket_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join("zlaunch.sock")
}

#[cfg(windows)]
pub fn get_socket_addr() -> &'static str {
    // Use a fixed localhost port for IPC on Windows
    "127.0.0.1:47392"
}
