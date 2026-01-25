//! Rendering implementation for LauncherView.

use gpui::{Context, Length, Window, div, image_cache, prelude::*, px, retain_all};
use gpui_component::list::List;
use gpui_component::{ActiveTheme, Icon, IconName};

use super::LauncherView;
use super::state::ViewMode;

impl gpui::Render for LauncherView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Clone theme to avoid borrow conflicts
        let theme = self.current_theme.clone();
        let config = crate::config::config();

        // Input prefix (icon based on mode and navigation state)
        let input_prefix = self.render_input_prefix(cx);

        // List content based on mode
        let list_content = self.render_list_content(window, cx);

        // Outer container - fullscreen with centered content
        let on_hide = self.on_hide.clone();
        div()
            .track_focus(&self.focus_handle)
            .key_context("LauncherView")
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::select_tab))
            .on_action(cx.listener(Self::select_tab_prev))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::go_back))
            .on_action(cx.listener(Self::switch_mode_next))
            .on_action(cx.listener(Self::switch_mode_prev))
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            // Click on backdrop to close
            .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, _cx| {
                on_hide();
            })
            // Inner launcher box - fixed width and height
            .child(
                div()
                    .id("launcher-panel")
                    .w(px(config.window_width))
                    .h(px(config.window_height))
                    .flex()
                    .flex_col()
                    .bg(if config.enable_transparency {
                        theme.window_background
                    } else {
                        theme.window_background.alpha(1.0)
                    })
                    .border_1()
                    .border_color(theme.window_border)
                    .rounded(theme.window_border_radius)
                    .overflow_hidden()
                    // Stop click propagation to backdrop
                    .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
                        cx.stop_propagation();
                    })
                    // Input section
                    .child(
                        div()
                            .w_full()
                            .px_2()
                            .py_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                gpui_component::input::Input::new(&self.input_state)
                                    .appearance(false)
                                    .cleanable(true)
                                    .prefix(input_prefix),
                            ),
                    )
                    // List content
                    .child(list_content),
            )
    }
}

impl LauncherView {
    /// Render the input prefix icon based on current mode and navigation state.
    fn render_input_prefix(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        match self.view_mode {
            ViewMode::Main => {
                // Icon based on current launcher mode
                let icon = match self.mode_state.current_mode() {
                    crate::config::LauncherMode::Combined => IconName::Search,
                    crate::config::LauncherMode::Applications => IconName::Search,
                    crate::config::LauncherMode::Windows => IconName::LayoutDashboard,
                    crate::config::LauncherMode::Actions => IconName::Settings,
                    crate::config::LauncherMode::Search => IconName::Globe,
                    crate::config::LauncherMode::Calculator => IconName::Search,
                    _ => IconName::Search,
                };
                Icon::new(icon)
                    .text_color(cx.theme().muted_foreground)
                    .mr_2()
                    .into_any_element()
            }
            ViewMode::EmojiPicker => {
                if self.navigated_into_submenu {
                    // Show back arrow when navigated from combined view
                    div()
                        .id("back-emoji")
                        .cursor_pointer()
                        .mr_2()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.exit_emoji_mode(window, cx);
                        }))
                        .child(
                            Icon::new(IconName::ArrowLeft).text_color(cx.theme().muted_foreground),
                        )
                        .into_any_element()
                } else {
                    // Show search icon in direct emoji mode
                    Icon::new(IconName::Search)
                        .text_color(cx.theme().muted_foreground)
                        .mr_2()
                        .into_any_element()
                }
            }
            ViewMode::ClipboardHistory => {
                if self.navigated_into_submenu {
                    div()
                        .id("back-clipboard")
                        .cursor_pointer()
                        .mr_2()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.exit_clipboard_mode(window, cx);
                        }))
                        .child(
                            Icon::new(IconName::ArrowLeft).text_color(cx.theme().muted_foreground),
                        )
                        .into_any_element()
                } else {
                    Icon::new(IconName::Copy)
                        .text_color(cx.theme().muted_foreground)
                        .mr_2()
                        .into_any_element()
                }
            }
            ViewMode::ThemePicker => {
                if self.navigated_into_submenu {
                    div()
                        .id("back-theme")
                        .cursor_pointer()
                        .mr_2()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.exit_theme_mode(window, cx);
                        }))
                        .child(
                            Icon::new(IconName::ArrowLeft).text_color(cx.theme().muted_foreground),
                        )
                        .into_any_element()
                } else {
                    Icon::new(IconName::Palette)
                        .text_color(cx.theme().muted_foreground)
                        .mr_2()
                        .into_any_element()
                }
            }
            ViewMode::AiResponse => {
                if self.navigated_into_submenu {
                    div()
                        .id("back-ai")
                        .cursor_pointer()
                        .mr_2()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.exit_ai_mode(window, cx);
                        }))
                        .child(
                            Icon::new(IconName::ArrowLeft).text_color(cx.theme().muted_foreground),
                        )
                        .into_any_element()
                } else {
                    Icon::new(IconName::Bot)
                        .text_color(cx.theme().muted_foreground)
                        .mr_2()
                        .into_any_element()
                }
            }
        }
    }

    /// Render the list content based on current view mode.
    fn render_list_content(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let theme = &self.current_theme;

        match self.view_mode {
            ViewMode::Main => image_cache(retain_all("app-icons"))
                .flex_1()
                .overflow_hidden()
                .py_2()
                .child(List::new(&self.list_state))
                .into_any_element(),
            ViewMode::EmojiPicker => {
                if let Some(emoji_state) = self.emoji_mode_handler.as_ref().map(|h| h.list_state())
                {
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .py_2()
                        .child(List::new(emoji_state))
                        .into_any_element()
                } else {
                    div().flex_1().into_any_element()
                }
            }
            ViewMode::ClipboardHistory => {
                if let Some(clipboard_state) =
                    self.clipboard_mode_handler.as_ref().map(|h| h.list_state())
                {
                    let selected_item =
                        clipboard_state.read(cx).delegate().selected_item().cloned();

                    div()
                        .flex_1()
                        .overflow_hidden()
                        .flex()
                        .flex_row()
                        // List column
                        .child(
                            div()
                                .w(Length::Definite(gpui::DefiniteLength::Fraction(0.5)))
                                .h_full()
                                .child(List::new(clipboard_state)),
                        )
                        // Separator
                        .child(
                            div()
                                .w(theme.layout.separator_width)
                                .h_full()
                                .bg(theme.window_border),
                        )
                        // Preview column
                        .child(
                            div()
                                .flex_1()
                                .h_full()
                                .bg(theme.item_background)
                                .rounded(theme.item_border_radius)
                                .overflow_hidden()
                                .child(self.render_clipboard_preview(selected_item.as_ref())),
                        )
                        .into_any_element()
                } else {
                    div().flex_1().into_any_element()
                }
            }
            ViewMode::ThemePicker => {
                if let Some(theme_state) = self.theme_mode_handler.as_ref().map(|h| h.list_state())
                {
                    image_cache(retain_all("theme-icons"))
                        .flex_1()
                        .overflow_hidden()
                        .py_2()
                        .child(List::new(theme_state))
                        .into_any_element()
                } else {
                    div().flex_1().into_any_element()
                }
            }
            ViewMode::AiResponse => {
                if let Some(ref handler) = self.ai_mode_handler {
                    div()
                        .flex_1()
                        .child(handler.view().render(window, cx))
                        .overflow_hidden()
                        .into_any_element()
                } else {
                    div().flex_1().into_any_element()
                }
            }
        }
    }
}
