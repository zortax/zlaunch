// ! Theme picker mode handler.
//!
//! Encapsulates all theme mode functionality:
//! - Creating and managing theme list state
//! - Handling theme preview on selection changes
//! - Reverting to previous theme on cancel
//! - Persisting theme selection on confirm

use crate::config::{list_all_themes_with_source, load_theme, save_theme_to_config};
use crate::items::ThemeItem;
use crate::ui::delegates::ThemeListDelegate;
use crate::ui::theme::LauncherTheme;
use gpui::{AppContext, Context, Entity, Subscription, Window};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::list::ListState;
use std::sync::Arc;

/// Handler for theme picker mode.
pub struct ThemeModeHandler {
    /// The theme list state
    list_state: Entity<ListState<ThemeListDelegate>>,
    /// The theme name that was active before entering theme mode
    previous_theme_name: String,
    /// Subscription to input changes (for filtering)
    _input_subscription: Subscription,
}

impl ThemeModeHandler {
    /// Create a new theme mode handler.
    ///
    /// # Parameters
    /// - `input_state`: The input field state
    /// - `current_theme_name`: The currently active theme name
    /// - `on_theme_change`: Callback to apply theme preview (called on selection change)
    /// - `on_confirm`: Callback when user confirms theme selection (Enter)
    /// - `on_cancel`: Callback when user cancels (ESC)
    /// - `window`: The window context
    /// - `cx`: The GPUI context
    pub fn new<T: 'static>(
        input_state: &Entity<InputState>,
        current_theme_name: String,
        on_theme_change: Arc<dyn Fn(LauncherTheme) + Send + Sync>,
        on_confirm: Arc<dyn Fn(String) + Send + Sync>,
        on_cancel: Arc<dyn Fn() + Send + Sync>,
        window: &mut Window,
        cx: &mut Context<T>,
    ) -> Self {
        // Load all available themes
        let themes_with_source = list_all_themes_with_source();
        let mut theme_items: Vec<ThemeItem> = themes_with_source
            .into_iter()
            .filter_map(|(name, source)| {
                load_theme(&name).map(|theme| ThemeItem::new(name, source, theme))
            })
            .collect();

        // Sort: current theme first, then alphabetically
        let current_theme_name_clone = current_theme_name.clone();
        theme_items.sort_by(|a, b| {
            if a.name == current_theme_name_clone {
                std::cmp::Ordering::Less
            } else if b.name == current_theme_name_clone {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        // Create delegate
        let mut delegate = ThemeListDelegate::new(theme_items);

        // Set up confirm callback (save theme and confirm)
        let on_confirm_clone = on_confirm.clone();
        delegate.set_on_confirm(move |theme_item| {
            // Save to config if possible
            if let Err(e) = save_theme_to_config(&theme_item.name) {
                tracing::warn!(%e, "Failed to save theme to config");
            }
            // Call confirm callback
            on_confirm_clone(theme_item.name.clone());
        });

        // Set up cancel callback
        delegate.set_on_cancel(move || {
            on_cancel();
        });

        // Create list state
        let list_state = cx.new(|cx| ListState::new(delegate, window, cx));

        // Subscribe to selection changes for live preview
        let _preview_subscription = cx.observe(&list_state, move |_this, list_state_entity, cx| {
            list_state_entity.update(cx, |state, _cx| {
                if let Some(selected_item) = state.delegate().selected_item() {
                    // Clone the theme and apply it
                    let theme = selected_item.theme.clone();
                    on_theme_change(theme);
                }
            });
        });

        // Subscribe to input for filtering
        let list_state_for_search = list_state.clone();
        let input_subscription = cx.subscribe(input_state, move |_this, input, event, cx| {
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
            previous_theme_name: current_theme_name,
            _input_subscription: input_subscription,
        }
    }

    /// Get the list state for rendering.
    pub fn list_state(&self) -> &Entity<ListState<ThemeListDelegate>> {
        &self.list_state
    }

    /// Get the previous theme name (for reverting on cancel).
    pub fn previous_theme_name(&self) -> &str {
        &self.previous_theme_name
    }

    /// Update input placeholder when entering theme mode.
    pub fn setup_input(
        input_state: &mut InputState,
        window: &mut Window,
        cx: &mut Context<InputState>,
    ) {
        input_state.set_value("", window, cx);
        input_state.set_placeholder("Search themes...", window, cx);
    }

    /// Restore input placeholder when exiting theme mode.
    pub fn restore_input(
        input_state: &mut InputState,
        window: &mut Window,
        cx: &mut Context<InputState>,
    ) {
        input_state.set_value("", window, cx);
        input_state.set_placeholder("Search applications...", window, cx);
    }
}
