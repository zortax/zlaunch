//! Action handlers for LauncherView.
//!
//! Handles confirm, cancel, and go_back actions.

use std::sync::Arc;

use gpui::{Context, Window};

use crate::clipboard::copy_to_clipboard;
use crate::compositor::Compositor;
use crate::config::LauncherMode;
use crate::desktop::launch_application;
use crate::items::{Executable, ListItem};

use super::state::ViewMode;
use super::{Cancel, Confirm, GoBack, LauncherView};

impl LauncherView {
    /// Handle confirming the selected item.
    pub fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        match self.view_mode {
            ViewMode::Main => {
                // Check if a submenu or AI item is selected
                if let Some(item) = self.list_state.read(cx).delegate().get_item_at(
                    self.list_state
                        .read(cx)
                        .delegate()
                        .selected_index()
                        .unwrap_or(0),
                ) {
                    match item {
                        ListItem::Submenu(submenu) => match submenu.id.as_str() {
                            "submenu-emojis" => {
                                self.navigated_into_submenu = true;
                                self.enter_emoji_mode(window, cx);
                                return;
                            }
                            "submenu-clipboard" => {
                                self.navigated_into_submenu = true;
                                self.enter_clipboard_mode(window, cx);
                                return;
                            }
                            "submenu-themes" => {
                                self.navigated_into_submenu = true;
                                self.enter_theme_mode(window, cx);
                                return;
                            }
                            _ => {}
                        },
                        ListItem::Ai(_) => {
                            self.navigated_into_submenu = true;
                            self.enter_ai_mode(window, cx);
                            return;
                        }
                        _ => {}
                    }
                }
                // Regular item confirmation
                self.list_state.update(cx, |state, _cx| {
                    state.delegate().do_confirm();
                });
            }
            ViewMode::EmojiPicker => {
                if let Some(emoji_state) = self.emoji_mode_handler.as_ref().map(|h| h.list_state())
                {
                    emoji_state.update(cx, |state, _cx| {
                        state.delegate().do_confirm();
                    });
                }
            }
            ViewMode::ClipboardHistory => {
                if let Some(clipboard_state) =
                    self.clipboard_mode_handler.as_ref().map(|h| h.list_state())
                {
                    clipboard_state.update(cx, |state, _cx| {
                        state.delegate().do_confirm();
                    });
                }
            }
            ViewMode::ThemePicker => {
                if let Some(theme_state) = self.theme_mode_handler.as_ref().map(|h| h.list_state())
                {
                    theme_state.update(cx, |state, _cx| {
                        state.delegate().do_confirm();
                    });
                }
                // Exit theme mode after confirming
                self.exit_theme_mode(window, cx);
            }
            ViewMode::AiResponse => {
                // If already in AI mode, then send a new prompt
                self.update_ai_mode(window, cx);
            }
        }
    }

    /// Handle cancel action.
    pub fn cancel(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
        match self.view_mode {
            ViewMode::Main => {
                self.list_state.update(cx, |state, _cx| {
                    state.delegate().do_cancel();
                });
            }
            _ => {
                // In subviews, cancel goes back
                self.go_back(&GoBack, window, cx);
            }
        }
    }

    /// Handle go back action.
    pub fn go_back(&mut self, _: &GoBack, window: &mut Window, cx: &mut Context<Self>) {
        // In direct mode (non-Combined), going back hides the launcher
        let is_direct_mode = !matches!(self.mode_state.current_mode(), LauncherMode::Combined);

        match self.view_mode {
            ViewMode::Main => {
                // Already at main, do nothing (or hide in direct mode)
                if is_direct_mode {
                    (self.on_hide)();
                }
            }
            ViewMode::EmojiPicker if is_direct_mode => {
                // In direct emoji mode, hide the launcher
                (self.on_hide)();
            }
            ViewMode::ClipboardHistory if is_direct_mode => {
                // In direct clipboard mode, hide the launcher
                (self.on_hide)();
            }
            ViewMode::ThemePicker if is_direct_mode => {
                // In direct theme mode, revert theme and hide
                crate::ui::theme::sync_theme_from_config();
                self.current_theme = crate::ui::theme::theme();
                (self.on_hide)();
            }
            ViewMode::EmojiPicker => {
                self.exit_emoji_mode(window, cx);
            }
            ViewMode::ClipboardHistory => {
                self.exit_clipboard_mode(window, cx);
            }
            ViewMode::ThemePicker => {
                self.exit_theme_mode(window, cx);
            }
            ViewMode::AiResponse => {
                self.exit_ai_mode(window, cx);
            }
        }
    }

    /// Handle confirming an item (static method for callbacks).
    pub fn handle_item_confirm(item: &ListItem, compositor: &Arc<dyn Compositor>) {
        match item {
            ListItem::Application(app) => {
                // Convert to DesktopEntry and launch
                let entry = crate::desktop::DesktopEntry::new(
                    app.id.clone(),
                    app.name.clone(),
                    app.exec.clone(),
                    None,
                    app.icon_path.clone(),
                    app.description.clone(),
                    vec![],
                    app.terminal,
                    app.desktop_path.clone(),
                );
                let _ = launch_application(&entry);
            }
            ListItem::Window(win) => {
                if let Err(e) = compositor.focus_window(&win.address) {
                    tracing::warn!(%e, "Failed to focus window");
                }
            }
            ListItem::Calculator(calc) => {
                if let Err(e) = copy_to_clipboard(calc.text_for_clipboard()) {
                    tracing::warn!(%e, "Failed to copy to clipboard");
                }
            }
            ListItem::Action(act) => {
                if let Err(e) = act.execute() {
                    tracing::warn!(%e, "Failed to execute action");
                }
            }
            ListItem::Search(search) => {
                if let Err(e) = search.execute() {
                    tracing::warn!(%e, "Failed to open search URL");
                }
            }
            ListItem::Submenu(submenu) => {
                // Submenu items are handled separately (e.g., enter_emoji_mode)
                tracing::debug!(id = %submenu.id, "Submenu selected");
            }
            ListItem::Ai(_ai) => {
                // AI items would trigger AI mode
                tracing::debug!("AI item selected");
            }
            ListItem::Theme(_theme) => {
                // Theme items are handled in theme mode
                tracing::debug!("Theme item selected");
            }
        }
    }
}
