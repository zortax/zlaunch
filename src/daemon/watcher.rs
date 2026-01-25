//! Background file watcher for automatic application reload.
//!
//! Watches XDG application directories for changes and sends
//! `ApplicationsChanged` events to the daemon event loop.

use std::time::Duration;

use tracing::{debug, error, info};

use crate::app::DaemonEvent;
use crate::desktop::watcher::ApplicationWatcher;

use super::init::load_application_items;

/// Run the watcher loop as an async task.
///
/// This should be spawned on the shared tokio runtime via `tokio_runtime::spawn()`.
pub async fn run_watcher_loop(event_tx: flume::Sender<DaemonEvent>) {
    let watcher = match ApplicationWatcher::new() {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to create application watcher: {}", e);
            return;
        }
    };

    info!("Application watcher started");

    loop {
        // Async wait for first event (flume works with tokio)
        let Ok(_event) = watcher.recv_async().await else {
            debug!("Watcher channel closed, exiting");
            return;
        };

        debug!("File watcher detected changes");

        // Debounce: wait for rapid changes to settle
        tokio::time::sleep(Duration::from_millis(500)).await;
        let _ = watcher.poll_events(); // Drain additional events

        // Reload applications
        let applications = load_application_items();
        info!("Reloaded {} applications", applications.len());
        if event_tx
            .send(DaemonEvent::ApplicationsChanged { applications })
            .is_err()
        {
            debug!("Event channel closed, watcher exiting");
            return;
        }
    }
}
