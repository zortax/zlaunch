//! Clipboard history mode handler.
//!
//! Encapsulates all clipboard mode functionality:
//! - Creating and managing clipboard list state
//! - Setting up input filtering
//! - Handling clipboard item selection and pasting

use crate::calculator::copy_to_clipboard;
use crate::clipboard::{data::search_items, ClipboardContent};
use crate::ui::delegates::ClipboardListDelegate;
use gpui::{AppContext, Context, Entity, Subscription, Window};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::list::ListState;
use std::sync::Arc;

/// Handler for clipboard history mode.
pub struct ClipboardModeHandler {
    /// The clipboard list state
    list_state: Entity<ListState<ClipboardListDelegate>>,
    /// Subscription to input changes (for filtering)
    _input_subscription: Subscription,
}

impl ClipboardModeHandler {
    /// Create a new clipboard mode handler.
    pub fn new<T: 'static>(
        input_state: &Entity<InputState>,
        on_hide: Arc<dyn Fn() + Send + Sync>,
        window: &mut Window,
        cx: &mut Context<T>,
    ) -> Self {
        // Create delegate with initial empty search
        let mut delegate = ClipboardListDelegate::new(search_items(""));

        // Set up confirm callback (copy item and hide)
        delegate.set_on_confirm(move |item| {
            let text = match &item.content {
                ClipboardContent::Text(t) => t.clone(),
                ClipboardContent::Image { .. } => return, // Can't paste images yet
                ClipboardContent::FilePaths(paths) => {
                    // Paste file paths as text
                    paths
                        .iter()
                        .filter_map(|p| p.to_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                ClipboardContent::RichText { plain, .. } => plain.clone(),
            };
            if let Err(e) = copy_to_clipboard(&text) {
                tracing::warn!(%e, "Failed to copy to clipboard");
            }
            on_hide();
        });

        // Create list state
        let list_state = cx.new(|cx| ListState::new(delegate, window, cx));

        // Subscribe to input for filtering
        let list_state_for_search = list_state.clone();
        let subscription = cx.subscribe(input_state, move |_this, input, event, cx| {
            if let InputEvent::Change = event {
                let query = input.read(cx).value().to_string();
                list_state_for_search.update(cx, |state, cx| {
                    state.delegate_mut().set_query(query);
                    cx.notify();
                });
            }
        });

        Self {
            list_state,
            _input_subscription: subscription,
        }
    }

    /// Get the list state for rendering.
    pub fn list_state(&self) -> &Entity<ListState<ClipboardListDelegate>> {
        &self.list_state
    }

    /// Update input placeholder when entering clipboard mode.
    pub fn setup_input(input_state: &mut InputState, window: &mut Window, cx: &mut Context<InputState>) {
        input_state.set_value("", window, cx);
        input_state.set_placeholder("Search clipboard history...", window, cx);
    }

    /// Restore input placeholder when exiting clipboard mode.
    pub fn restore_input(input_state: &mut InputState, window: &mut Window, cx: &mut Context<InputState>) {
        input_state.set_value("", window, cx);
        input_state.set_placeholder("Search applications...", window, cx);
    }
}
