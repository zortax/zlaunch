//! Mode switching and management for LauncherView.
//!
//! Handles entering/exiting different modes (emoji, clipboard, AI, theme)
//! and switching between launcher modes.

use std::sync::Arc;

use gpui::{Context, IntoElement, Window};

use crate::config::LauncherMode;
use crate::ui::delegates::ItemListDelegate;
use crate::ui::modes::{
    AiModeHandler, ClipboardModeHandler, EmojiModeHandler, ThemeModeHandler,
};
use crate::ui::theme::LauncherTheme;
use gpui_component::list::ListState;

use super::state::ViewMode;
use super::{LauncherView, SwitchModeNext, SwitchModePrev};

impl LauncherView {
    /// Enter emoji picker mode.
    pub fn enter_emoji_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Create emoji mode handler
        let handler = EmojiModeHandler::new(&self.input_state, self.on_hide.clone(), window, cx);

        // Update input
        self.input_state.update(cx, |input, cx| {
            EmojiModeHandler::setup_input(input, window, cx);
        });

        self.emoji_mode_handler = Some(handler);
        self.view_mode = ViewMode::EmojiPicker;
        cx.notify();
    }

    /// Exit emoji picker mode.
    pub fn exit_emoji_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = ViewMode::Main;
        self.emoji_mode_handler = None;
        self.navigated_into_submenu = false;

        self.reset_search(window, cx);
        cx.notify();
    }

    /// Enter clipboard history mode.
    pub fn enter_clipboard_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Create clipboard mode handler
        let handler =
            ClipboardModeHandler::new(&self.input_state, self.on_hide.clone(), window, cx);

        // Update input
        self.input_state.update(cx, |input, cx| {
            ClipboardModeHandler::setup_input(input, window, cx);
        });

        self.clipboard_mode_handler = Some(handler);
        self.view_mode = ViewMode::ClipboardHistory;
        cx.notify();
    }

    /// Exit clipboard history mode.
    pub fn exit_clipboard_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = ViewMode::Main;
        self.clipboard_mode_handler = None;
        self.navigated_into_submenu = false;

        self.reset_search(window, cx);
        cx.notify();
    }

    /// Enter AI response mode.
    pub fn enter_ai_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Get the AI query from the selected item
        let selected_item = self.list_state.read(cx).delegate().get_item_at(
            self.list_state
                .read(cx)
                .delegate()
                .selected_index()
                .unwrap_or(0),
        );
        let query = if let Some(crate::items::ListItem::Ai(ai_item)) = selected_item {
            ai_item.query.clone()
        } else {
            return;
        };

        // Create AI mode handler (it starts streaming internally)
        let entity = cx.entity().downgrade();
        let handler = AiModeHandler::new(query.clone(), entity, cx);

        if let Some(handler) = handler {
            self.ai_mode_handler = Some(handler);

            // Update the input
            self.input_state.update(cx, |input, cx| {
                AiModeHandler::setup_input(input, window, cx);
            });

            // Switch to AI response mode
            self.view_mode = ViewMode::AiResponse;
            cx.notify();
        }
    }

    /// Update AI response mode with a new prompt.
    pub fn update_ai_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Get the AI query from the selected item
        let selected_item = self.list_state.read(cx).delegate().get_item_at(
            self.list_state
                .read(cx)
                .delegate()
                .selected_index()
                .unwrap_or(0),
        );
        let query = if let Some(crate::items::ListItem::Ai(ai_item)) = selected_item {
            ai_item.query.clone()
        } else {
            return;
        };

        // Clean the input
        self.input_state.update(cx, |input, cx| {
            AiModeHandler::clear_input(input, window, cx);
        });

        // Send a new prompt to the AI mode handler
        if let Some(handler) = &mut self.ai_mode_handler {
            handler.send_message(query, cx.weak_entity(), cx);
        }
    }

    /// Exit AI response mode and return to main view.
    pub fn exit_ai_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = ViewMode::Main;
        self.ai_mode_handler = None;
        self.navigated_into_submenu = false;

        self.reset_search(window, cx);
        cx.notify();
    }

    /// Enter theme picker mode.
    pub fn enter_theme_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current_theme_name = self.current_theme.name.clone();

        // Simple no-op callbacks (theme preview happens via observation below)
        let on_theme_change = Arc::new(move |_theme: LauncherTheme| {});
        let on_confirm = Arc::new(move |_theme_name: String| {});
        let on_cancel = Arc::new(move || {});

        let handler = ThemeModeHandler::new(
            &self.input_state,
            current_theme_name.clone(),
            on_theme_change,
            on_confirm,
            on_cancel,
            window,
            cx,
        );

        // Subscribe to theme list state changes for live preview
        let theme_list_state = handler.list_state().clone();
        let subscription = cx.observe(&theme_list_state, |launcher, list_state, cx| {
            // Update current_theme when selection changes
            list_state.update(cx, |state, _cx| {
                if let Some(selected_item) = state.delegate().selected_item() {
                    let new_theme = selected_item.theme.clone();
                    launcher.current_theme = new_theme.clone();
                    // Update global theme for live preview
                    crate::ui::theme::set_theme(new_theme);
                }
            });
            cx.notify();
        });

        self.input_state.update(cx, |input, cx| {
            ThemeModeHandler::setup_input(input, window, cx);
        });

        self.theme_mode_handler = Some(handler);
        self._theme_preview_subscription = Some(subscription);
        self.view_mode = ViewMode::ThemePicker;
        cx.notify();
    }

    /// Exit theme picker mode.
    pub fn exit_theme_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = ViewMode::Main;
        self.theme_mode_handler = None;
        self._theme_preview_subscription = None;
        self.navigated_into_submenu = false;

        // Reload the configured theme and update the global cache
        crate::ui::theme::sync_theme_from_config();
        self.current_theme = crate::ui::theme::theme();

        self.reset_search(window, cx);
        cx.notify();
    }

    /// Render clipboard preview panel.
    pub fn render_clipboard_preview(
        &self,
        item: Option<&crate::clipboard::ClipboardItem>,
    ) -> impl IntoElement {
        crate::ui::views::clipboard_rendering::render_preview_panel(item)
    }

    /// Switch to the next mode.
    pub fn switch_mode_next(
        &mut self,
        _: &SwitchModeNext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.mode_state.has_multiple_modes() {
            return;
        }
        self.mode_state.next_mode();
        self.apply_current_mode(window, cx);
    }

    /// Switch to the previous mode.
    pub fn switch_mode_prev(
        &mut self,
        _: &SwitchModePrev,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.mode_state.has_multiple_modes() {
            return;
        }
        self.mode_state.prev_mode();
        self.apply_current_mode(window, cx);
    }

    /// Apply the current mode by switching view modes and setting up handlers.
    pub fn apply_current_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Clean up current mode handlers
        self.cleanup_mode_handlers(window, cx);

        // Reset navigation flag - mode switching is not navigation
        self.navigated_into_submenu = false;

        // Set new view mode and initialize handler
        match self.mode_state.current_mode() {
            LauncherMode::Combined => {
                self.recreate_delegate_for_mode(window, cx);
                self.view_mode = ViewMode::Main;
                self.reset_search(window, cx);
            }
            LauncherMode::Emojis => {
                self.enter_emoji_mode(window, cx);
            }
            LauncherMode::Clipboard => {
                self.enter_clipboard_mode(window, cx);
            }
            LauncherMode::Themes => {
                self.enter_theme_mode(window, cx);
            }
            _ => {
                // For other modes (Applications, Windows, Actions, Search, Calculator),
                // recreate delegate with filtered modules and use Main view
                self.recreate_delegate_for_mode(window, cx);
                self.view_mode = ViewMode::Main;
                self.reset_search(window, cx);
            }
        }

        cx.notify();
    }

    /// Recreate the main delegate for the current mode with appropriate module filtering.
    pub fn recreate_delegate_for_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let modules = Self::modules_for_mode(self.mode_state.current_mode());

        // Create new delegate with filtered modules
        let mut delegate = ItemListDelegate::new(self.original_items.clone(), modules);

        // Set up callbacks
        let on_hide = self.on_hide.clone();
        let compositor = self.compositor.clone();
        delegate.set_on_confirm(move |item| {
            Self::handle_item_confirm(item, &compositor);
            on_hide();
        });

        let on_hide_for_cancel = self.on_hide.clone();
        delegate.set_on_cancel(move || on_hide_for_cancel());

        // Update the list state with the new delegate
        self.list_state.update(cx, |state, cx| {
            *state = ListState::new(delegate, window, cx);
        });
    }

    /// Clean up all mode handlers.
    pub fn cleanup_mode_handlers(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.emoji_mode_handler = None;
        self.clipboard_mode_handler = None;
        self.ai_mode_handler = None;
        self.theme_mode_handler = None;
        self._theme_preview_subscription = None;
    }
}
