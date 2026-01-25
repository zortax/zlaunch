//! Shared tokio runtime for use with GPUI.
//!
//! Provides a multi-threaded tokio runtime stored as a GPUI global,
//! allowing async tokio tasks to be spawned from anywhere in the app.

use gpui::{App, Global};
use std::future::Future;
use tokio::runtime::{Handle, Runtime};

/// Holds the tokio runtime.
enum RuntimeHolder {
    Owned(Runtime),
}

impl RuntimeHolder {
    fn handle(&self) -> Handle {
        match self {
            Self::Owned(rt) => rt.handle().clone(),
        }
    }
}

/// Global tokio runtime state.
struct GlobalTokio {
    runtime: RuntimeHolder,
}

impl Global for GlobalTokio {}

/// Initialize the shared tokio runtime.
/// Call this once during daemon startup, before any tokio tasks are spawned.
pub fn init(cx: &mut App) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to initialize tokio runtime");

    cx.set_global(GlobalTokio {
        runtime: RuntimeHolder::Owned(runtime),
    });
}

/// Get a handle to the shared tokio runtime.
pub fn handle(cx: &App) -> Handle {
    cx.global::<GlobalTokio>().runtime.handle()
}

/// Spawn a future on the shared tokio runtime.
pub fn spawn<F>(cx: &App, future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    handle(cx).spawn(future)
}
