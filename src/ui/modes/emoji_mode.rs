//! Emoji picker mode handler.
//!
//! Encapsulates all emoji mode functionality:
//! - Creating and managing emoji grid state
//! - Setting up input filtering
//! - Handling emoji selection and copying

use crate::calculator::copy_to_clipboard;
use crate::emoji::all_emojis;
use crate::ui::delegates::EmojiGridDelegate;
use gpui::{AppContext, Context, Entity, Subscription, Window};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::list::ListState;
use std::sync::Arc;

/// Handler for emoji picker mode.
pub struct EmojiModeHandler {
    /// The emoji grid list state
    list_state: Entity<ListState<EmojiGridDelegate>>,
    /// Subscription to input changes (for filtering)
    _input_subscription: Subscription,
}

impl EmojiModeHandler {
    /// Create a new emoji mode handler.
    pub fn new<T: 'static>(
        input_state: &Entity<InputState>,
        on_hide: Arc<dyn Fn() + Send + Sync>,
        window: &mut Window,
        cx: &mut Context<T>,
    ) -> Self {
        // Create delegate with theme-based column count
        let mut delegate = EmojiGridDelegate::new(all_emojis().to_vec(), crate::ui::theme::theme().emoji.columns);

        // Set up confirm callback (copy emoji and hide)
        delegate.set_on_confirm(move |emoji| {
            if let Err(e) = copy_to_clipboard(&emoji.emoji) {
                tracing::warn!(%e, "Failed to copy emoji to clipboard");
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
    pub fn list_state(&self) -> &Entity<ListState<EmojiGridDelegate>> {
        &self.list_state
    }

    /// Update input placeholder when entering emoji mode.
    pub fn setup_input(input_state: &mut InputState, window: &mut Window, cx: &mut Context<InputState>) {
        input_state.set_value("", window, cx);
        input_state.set_placeholder("Search emojis...", window, cx);
    }

    /// Restore input placeholder when exiting emoji mode.
    pub fn restore_input(input_state: &mut InputState, window: &mut Window, cx: &mut Context<InputState>) {
        input_state.set_value("", window, cx);
        input_state.set_placeholder("Search applications...", window, cx);
    }
}
