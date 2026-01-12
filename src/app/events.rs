//! Event types for daemon communication.

use tokio::sync::oneshot;

/// Response type for IPC operations.
pub type IpcResponse = Result<(), String>;

/// Events that the UI can send to the daemon.
#[derive(Debug, Clone, Copy)]
pub enum WindowEvent {
    RequestHide,
}

/// Unified event type for the daemon event loop.
/// Combines IPC commands and window events into a single channel.
pub enum DaemonEvent {
    /// Window event from the UI
    Window(WindowEvent),

    /// Show the launcher window
    Show {
        response_tx: oneshot::Sender<IpcResponse>,
    },

    /// Hide the launcher window
    Hide {
        response_tx: oneshot::Sender<IpcResponse>,
    },

    /// Toggle the launcher window visibility
    Toggle {
        response_tx: oneshot::Sender<IpcResponse>,
    },

    /// Quit the daemon
    Quit {
        response_tx: oneshot::Sender<IpcResponse>,
    },

    /// Set the active theme
    SetTheme {
        name: String,
        response_tx: oneshot::Sender<IpcResponse>,
    },

    /// Reload the daemon (restart the process)
    Reload {
        response_tx: oneshot::Sender<IpcResponse>,
    },
}

impl From<WindowEvent> for DaemonEvent {
    fn from(event: WindowEvent) -> Self {
        Self::Window(event)
    }
}

/// Async sender for daemon events.
pub type DaemonEventSender = flume::Sender<DaemonEvent>;

/// Async receiver for daemon events.
pub type DaemonEventReceiver = flume::Receiver<DaemonEvent>;

/// Create an unbounded async channel for daemon events.
pub fn create_daemon_channel() -> (DaemonEventSender, DaemonEventReceiver) {
    flume::unbounded()
}

// Legacy types for backwards compatibility during refactoring
pub type EventSender = flume::Sender<DaemonEvent>;
pub type EventReceiver = flume::Receiver<DaemonEvent>;

pub fn create_event_channel() -> (EventSender, EventReceiver) {
    create_daemon_channel()
}
