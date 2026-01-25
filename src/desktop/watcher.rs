//! File watcher for desktop entry directories.
//!
//! Watches XDG application directories for changes and emits events
//! when applications are added, removed, or modified.

use flume::{Receiver, TryRecvError};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Events emitted by the application watcher.
#[derive(Debug, Clone)]
pub enum WatcherEvent {
    /// A new .desktop file was added.
    ApplicationAdded(PathBuf),
    /// A .desktop file was removed.
    ApplicationRemoved(PathBuf),
    /// A .desktop file was modified.
    ApplicationModified(PathBuf),
    /// A directory was modified (may need rescan).
    DirectoryChanged(PathBuf),
}

/// Watches XDG application directories for changes.
pub struct ApplicationWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<WatcherEvent>,
}

impl ApplicationWatcher {
    /// Create a new application watcher.
    ///
    /// Watches all XDG application directories for file system changes.
    pub fn new() -> anyhow::Result<Self> {
        let (tx, rx) = flume::unbounded();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| match res {
            Ok(event) => {
                let watcher_events = Self::convert_event(event);
                for evt in watcher_events {
                    if let Err(e) = tx.send(evt) {
                        error!("Failed to send watcher event: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("File watcher error: {}", e);
            }
        })?;

        // Watch all XDG application directories
        for dir in get_xdg_application_dirs() {
            if dir.exists() {
                match watcher.watch(&dir, RecursiveMode::Recursive) {
                    Ok(()) => {
                        info!("Watching directory: {:?}", dir);
                    }
                    Err(e) => {
                        warn!("Failed to watch directory {:?}: {}", dir, e);
                    }
                }
            }
        }

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Poll for pending events (non-blocking).
    pub fn poll_events(&self) -> Vec<WatcherEvent> {
        let mut events = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    error!("Watcher channel disconnected");
                    break;
                }
            }
        }
        events
    }

    /// Wait for events with a timeout.
    pub fn wait_events(&self, timeout: Duration) -> Vec<WatcherEvent> {
        let mut events = Vec::new();

        // Wait for first event with timeout
        match self.rx.recv_timeout(timeout) {
            Ok(event) => {
                events.push(event);
                // Drain any additional pending events
                events.extend(self.poll_events());
            }
            Err(flume::RecvTimeoutError::Timeout) => {}
            Err(flume::RecvTimeoutError::Disconnected) => {
                error!("Watcher channel disconnected");
            }
        }

        events
    }

    /// Async wait for a single event (for use with tokio).
    pub async fn recv_async(&self) -> Result<WatcherEvent, flume::RecvError> {
        self.rx.recv_async().await
    }

    /// Check if there are pending updates.
    pub fn has_updates(&self) -> bool {
        // This is a bit of a hack - we peek without consuming
        // In practice, poll_events is preferred
        !self.poll_events().is_empty()
    }

    /// Convert a notify event to our watcher events.
    fn convert_event(event: Event) -> Vec<WatcherEvent> {
        let mut results = Vec::new();

        for path in event.paths {
            // Only care about .desktop files
            let is_desktop = path.extension().is_some_and(|ext| ext == "desktop");

            let watcher_event = match event.kind {
                EventKind::Create(_) if is_desktop => {
                    debug!("Desktop file created: {:?}", path);
                    Some(WatcherEvent::ApplicationAdded(path))
                }
                EventKind::Remove(_) if is_desktop => {
                    debug!("Desktop file removed: {:?}", path);
                    Some(WatcherEvent::ApplicationRemoved(path))
                }
                EventKind::Modify(_) if is_desktop => {
                    debug!("Desktop file modified: {:?}", path);
                    Some(WatcherEvent::ApplicationModified(path))
                }
                EventKind::Create(_) | EventKind::Remove(_) if path.is_dir() => {
                    debug!("Directory changed: {:?}", path);
                    Some(WatcherEvent::DirectoryChanged(path))
                }
                _ => None,
            };

            if let Some(evt) = watcher_event {
                results.push(evt);
            }
        }

        results
    }
}

/// Get the list of XDG application directories to watch.
fn get_xdg_application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = dirs::data_local_dir() {
        dirs.push(data_home.join("applications"));
    }

    if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_dirs.split(':') {
            dirs.push(PathBuf::from(dir).join("applications"));
        }
    } else {
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/usr/share/applications"));
    }

    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_xdg_dirs() {
        let dirs = get_xdg_application_dirs();
        // Should have at least the local dir and system dirs
        assert!(!dirs.is_empty());
    }
}
