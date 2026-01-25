//! Navigation methods for LauncherView.
//!
//! Handles up/down/tab navigation across all view modes.

use gpui::{Context, ScrollStrategy, Window};
use gpui_component::IndexPath;

use super::state::ViewMode;
use super::{LauncherView, SelectNext, SelectPrev, SelectTab, SelectTabPrev};

impl LauncherView {
    /// Navigate to the next item.
    pub fn select_next(&mut self, _: &SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        match self.view_mode {
            ViewMode::Main => {
                self.list_state.update(cx, |state, cx| {
                    state.delegate_mut().select_down();
                    if let Some(idx) = state.delegate().selected_index()
                        && let Some(index_path) = state.delegate().global_to_index_path(idx)
                    {
                        // Update the List's internal selection
                        state.set_selected_index(Some(index_path), window, cx);
                        state.scroll_to_item(index_path, ScrollStrategy::Top, window, cx);
                    }
                    cx.notify();
                });
            }
            ViewMode::EmojiPicker => {
                if let Some(emoji_state) = self.emoji_mode_handler.as_ref().map(|h| h.list_state())
                {
                    emoji_state.update(cx, |state, cx| {
                        state.delegate_mut().select_down();
                        if let Some(row) = state.delegate().selected_row() {
                            state.scroll_to_item(
                                IndexPath::new(row),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::ClipboardHistory => {
                if let Some(clipboard_state) =
                    self.clipboard_mode_handler.as_ref().map(|h| h.list_state())
                {
                    clipboard_state.update(cx, |state, cx| {
                        state.delegate_mut().select_down();
                        if let Some(idx) = state.delegate().selected_index() {
                            state.scroll_to_item(
                                IndexPath::new(idx),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::ThemePicker => {
                if let Some(theme_state) = self.theme_mode_handler.as_ref().map(|h| h.list_state())
                {
                    theme_state.update(cx, |state, cx| {
                        state.delegate_mut().select_down();
                        if let Some(idx) = state.delegate().selected_index() {
                            state.scroll_to_item(
                                IndexPath::new(idx),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::AiResponse => {
                // No navigation in AI response mode
            }
        }
    }

    /// Navigate to the previous item.
    pub fn select_prev(&mut self, _: &SelectPrev, window: &mut Window, cx: &mut Context<Self>) {
        match self.view_mode {
            ViewMode::Main => {
                self.list_state.update(cx, |state, cx| {
                    state.delegate_mut().select_up();
                    if let Some(idx) = state.delegate().selected_index()
                        && let Some(index_path) = state.delegate().global_to_index_path(idx)
                    {
                        // Update the List's internal selection
                        state.set_selected_index(Some(index_path), window, cx);
                        state.scroll_to_item(index_path, ScrollStrategy::Top, window, cx);
                    }
                    cx.notify();
                });
            }
            ViewMode::EmojiPicker => {
                if let Some(emoji_state) = self.emoji_mode_handler.as_ref().map(|h| h.list_state())
                {
                    emoji_state.update(cx, |state, cx| {
                        state.delegate_mut().select_up();
                        if let Some(row) = state.delegate().selected_row() {
                            state.scroll_to_item(
                                IndexPath::new(row),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::ClipboardHistory => {
                if let Some(clipboard_state) =
                    self.clipboard_mode_handler.as_ref().map(|h| h.list_state())
                {
                    clipboard_state.update(cx, |state, cx| {
                        state.delegate_mut().select_up();
                        if let Some(idx) = state.delegate().selected_index() {
                            state.scroll_to_item(
                                IndexPath::new(idx),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::ThemePicker => {
                if let Some(theme_state) = self.theme_mode_handler.as_ref().map(|h| h.list_state())
                {
                    theme_state.update(cx, |state, cx| {
                        state.delegate_mut().select_up();
                        if let Some(idx) = state.delegate().selected_index() {
                            state.scroll_to_item(
                                IndexPath::new(idx),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::AiResponse => {
                // No navigation in AI response mode
            }
        }
    }

    /// Tab moves to next item linearly with wrapping.
    pub fn select_tab(&mut self, _: &SelectTab, window: &mut Window, cx: &mut Context<Self>) {
        match self.view_mode {
            ViewMode::Main => {
                self.list_state.update(cx, |state, cx| {
                    let delegate = state.delegate_mut();
                    let count = delegate.filtered_count();
                    if count == 0 {
                        return;
                    }
                    let current = delegate.selected_index().unwrap_or(0);
                    let next = if current + 1 >= count { 0 } else { current + 1 };
                    delegate.set_selected(next);

                    if let Some(index_path) = delegate.global_to_index_path(next) {
                        state.set_selected_index(Some(index_path), window, cx);
                        state.scroll_to_item(index_path, ScrollStrategy::Top, window, cx);
                    }
                    cx.notify();
                });
            }
            ViewMode::EmojiPicker => {
                if let Some(emoji_state) = self.emoji_mode_handler.as_ref().map(|h| h.list_state())
                {
                    emoji_state.update(cx, |state, cx| {
                        state.delegate_mut().select_right();
                        if let Some(row) = state.delegate().selected_row() {
                            state.scroll_to_item(
                                IndexPath::new(row),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::ClipboardHistory => {
                if let Some(clipboard_state) =
                    self.clipboard_mode_handler.as_ref().map(|h| h.list_state())
                {
                    clipboard_state.update(cx, |state, cx| {
                        state.delegate_mut().select_down();
                        if let Some(idx) = state.delegate().selected_index() {
                            state.scroll_to_item(
                                IndexPath::new(idx),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::ThemePicker => {
                if let Some(theme_state) = self.theme_mode_handler.as_ref().map(|h| h.list_state())
                {
                    theme_state.update(cx, |state, cx| {
                        state.delegate_mut().select_down();
                        if let Some(idx) = state.delegate().selected_index() {
                            state.scroll_to_item(
                                IndexPath::new(idx),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::AiResponse => {
                // No navigation in AI response mode
            }
        }
    }

    /// Shift+Tab moves to previous item linearly with wrapping.
    pub fn select_tab_prev(
        &mut self,
        _: &SelectTabPrev,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match self.view_mode {
            ViewMode::Main => {
                self.list_state.update(cx, |state, cx| {
                    let delegate = state.delegate_mut();
                    let count = delegate.filtered_count();
                    if count == 0 {
                        return;
                    }
                    let current = delegate.selected_index().unwrap_or(0);
                    let prev = if current == 0 { count - 1 } else { current - 1 };
                    delegate.set_selected(prev);

                    if let Some(index_path) = delegate.global_to_index_path(prev) {
                        state.set_selected_index(Some(index_path), window, cx);
                        state.scroll_to_item(index_path, ScrollStrategy::Top, window, cx);
                    }
                    cx.notify();
                });
            }
            ViewMode::EmojiPicker => {
                if let Some(emoji_state) = self.emoji_mode_handler.as_ref().map(|h| h.list_state())
                {
                    emoji_state.update(cx, |state, cx| {
                        state.delegate_mut().select_left();
                        if let Some(row) = state.delegate().selected_row() {
                            state.scroll_to_item(
                                IndexPath::new(row),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::ClipboardHistory => {
                if let Some(clipboard_state) =
                    self.clipboard_mode_handler.as_ref().map(|h| h.list_state())
                {
                    clipboard_state.update(cx, |state, cx| {
                        state.delegate_mut().select_up();
                        if let Some(idx) = state.delegate().selected_index() {
                            state.scroll_to_item(
                                IndexPath::new(idx),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::ThemePicker => {
                if let Some(theme_state) = self.theme_mode_handler.as_ref().map(|h| h.list_state())
                {
                    theme_state.update(cx, |state, cx| {
                        state.delegate_mut().select_up();
                        if let Some(idx) = state.delegate().selected_index() {
                            state.scroll_to_item(
                                IndexPath::new(idx),
                                ScrollStrategy::Top,
                                window,
                                cx,
                            );
                        }
                        cx.notify();
                    });
                }
            }
            ViewMode::AiResponse => {
                // No navigation in AI response mode
            }
        }
    }
}
