use crate::clipboard::copy_to_clipboard;
use crate::compositor::Compositor;
use crate::desktop::launch_application;
use crate::items::{Executable, ListItem};
use crate::ui::delegates::ItemListDelegate;
use crate::ui::modes::{
    AiModeAccess, AiModeHandler, ClipboardModeHandler, EmojiModeHandler, ThemeModeHandler,
};
use crate::ui::theme::LauncherTheme;
use gpui::{
    App, Context, Entity, FocusHandle, Focusable, KeyBinding, Length, ScrollStrategy, Window,
    actions, div, image_cache, prelude::*, px, retain_all,
};
use gpui_component::input::InputState;
use gpui_component::list::{List, ListState};
use gpui_component::{ActiveTheme, Icon, IconName, IndexPath};
use std::sync::Arc;

actions!(
    launcher,
    [
        SelectNext,
        SelectPrev,
        SelectTab,
        SelectTabPrev,
        Confirm,
        Cancel,
        GoBack
    ]
);

/// The current view mode of the launcher.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    /// Main launcher view showing apps, windows, commands.
    #[default]
    Main,
    /// Emoji picker grid view.
    EmojiPicker,
    /// Clipboard history view.
    ClipboardHistory,
    /// AI response streaming view.
    AiResponse,
    /// Theme picker view.
    ThemePicker,
}

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", SelectPrev, Some("LauncherView")),
        KeyBinding::new("down", SelectNext, Some("LauncherView")),
        KeyBinding::new("tab", SelectTab, Some("LauncherView")),
        KeyBinding::new("shift-tab", SelectTabPrev, Some("LauncherView")),
        KeyBinding::new("enter", Confirm, Some("LauncherView")),
        KeyBinding::new("escape", Cancel, Some("LauncherView")),
        KeyBinding::new("backspace", GoBack, Some("LauncherView")),
    ]);
}

/// The main launcher view.
pub struct LauncherView {
    /// Current view mode
    view_mode: ViewMode,
    /// Main list state
    list_state: Entity<ListState<ItemListDelegate>>,
    /// Emoji mode handler (created on demand)
    emoji_mode_handler: Option<EmojiModeHandler>,
    /// Clipboard mode handler (created on demand)
    clipboard_mode_handler: Option<ClipboardModeHandler>,
    /// AI mode handler (created on demand)
    ai_mode_handler: Option<AiModeHandler>,
    /// Theme mode handler (created on demand)
    theme_mode_handler: Option<ThemeModeHandler>,
    /// Current theme (for live preview)
    current_theme: LauncherTheme,
    /// Theme preview subscription
    _theme_preview_subscription: Option<gpui::Subscription>,
    /// Input state
    input_state: Entity<InputState>,
    /// Focus handle
    focus_handle: FocusHandle,
    /// Callback to hide the launcher
    on_hide: Arc<dyn Fn() + Send + Sync>,
}

impl LauncherView {
    /// Create a new launcher view.
    pub fn new(
        items: Vec<ListItem>,
        compositor: Arc<dyn Compositor>,
        on_hide: impl Fn() + Send + Sync + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let on_hide = Arc::new(on_hide);

        // Create main delegate with callbacks
        let mut delegate = ItemListDelegate::new(items);
        let on_hide_for_confirm = on_hide.clone();
        let compositor_for_confirm = compositor.clone();

        delegate.set_on_confirm(move |item| {
            Self::handle_item_confirm(item, &compositor_for_confirm);
            on_hide_for_confirm();
        });

        let on_hide_for_cancel = on_hide.clone();
        delegate.set_on_cancel(move || on_hide_for_cancel());

        let list_state = cx.new(|cx| ListState::new(delegate, window, cx));

        // Create input state
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search applications..."));

        // Subscribe to input changes
        let list_state_for_subscribe = list_state.clone();
        cx.subscribe(&input_state, move |_this, input, event, cx| {
            if let gpui_component::input::InputEvent::Change = event {
                let text = input.read(cx).value().to_string();
                // Update the delegate's query directly (synchronous filtering)
                list_state_for_subscribe.update(cx, |state, cx| {
                    state.delegate_mut().set_query(text);
                    cx.notify();
                });
            }
        })
        .detach();

        let focus_handle = cx.focus_handle();

        // Hide when the view loses focus
        let on_hide_for_blur = on_hide.clone();
        cx.on_blur(&focus_handle, window, move |_this, _window, _cx| {
            on_hide_for_blur();
        })
        .detach();

        Self {
            view_mode: ViewMode::Main,
            list_state,
            emoji_mode_handler: None,
            clipboard_mode_handler: None,
            ai_mode_handler: None,
            theme_mode_handler: None,
            current_theme: crate::config::load_configured_theme(),
            _theme_preview_subscription: None,
            input_state,
            focus_handle,
            on_hide,
        }
    }

    /// Refresh the current theme from the global state.
    /// Called when the theme is changed via IPC while the window is open.
    pub fn refresh_theme(&mut self, cx: &mut Context<Self>) {
        self.current_theme = crate::ui::theme::theme();
        cx.notify();
    }

    /// Handle confirming an item.
    fn handle_item_confirm(item: &ListItem, compositor: &Arc<dyn Compositor>) {
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

    /// Focus the launcher input.
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |input: &mut InputState, cx| {
            input.focus(window, cx);
        });
    }

    /// Reset search to empty state.
    pub fn reset_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.update(cx, |list_state, _cx| {
            list_state.delegate_mut().clear_query();
        });
        self.input_state.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
    }

    /// Enter emoji picker mode.
    fn enter_emoji_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
    fn exit_emoji_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = ViewMode::Main;
        self.emoji_mode_handler = None;

        self.reset_search(window, cx);
        self.input_state.update(cx, |input, cx| {
            EmojiModeHandler::restore_input(input, window, cx);
        });
        cx.notify();
    }

    /// Enter clipboard history mode.
    fn enter_clipboard_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
    fn exit_clipboard_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = ViewMode::Main;
        self.clipboard_mode_handler = None;

        self.reset_search(window, cx);
        self.input_state.update(cx, |input, cx| {
            ClipboardModeHandler::restore_input(input, window, cx);
        });
        cx.notify();
    }

    /// Enter AI response mode.
    fn enter_ai_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Get the AI query from the selected item
        let selected_item = self.list_state.read(cx).delegate().get_item_at(
            self.list_state
                .read(cx)
                .delegate()
                .selected_index()
                .unwrap_or(0),
        );
        let query = if let Some(ListItem::Ai(ai_item)) = selected_item {
            ai_item.query.clone()
        } else {
            return;
        };

        // Create AI mode handler (it starts streaming internally)
        let entity = cx.entity().downgrade();
        let handler = AiModeHandler::new(query.clone(), entity, cx);

        if let Some(handler) = handler {
            self.ai_mode_handler = Some(handler);

            // Update input to show just the query
            self.input_state.update(cx, |input, cx| {
                AiModeHandler::update_input(&query, input, window, cx);
            });

            // Switch to AI response mode
            self.view_mode = ViewMode::AiResponse;
            cx.notify();
        }
    }

    /// Exit AI response mode and return to main view.
    fn exit_ai_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = ViewMode::Main;

        // Clean up AI mode handler
        self.ai_mode_handler = None;

        // Clear search and reset placeholder
        self.input_state.update(cx, |input, cx| {
            AiModeHandler::clear_input(input, window, cx);
        });
        self.list_state.update(cx, |list_state, _cx| {
            list_state.delegate_mut().clear_query();
        });
        cx.notify();
    }

    /// Enter theme picker mode.
    fn enter_theme_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
    fn exit_theme_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = ViewMode::Main;
        self.theme_mode_handler = None;
        self._theme_preview_subscription = None;

        // Reload the configured theme and update the global cache
        crate::ui::theme::sync_theme_from_config();
        self.current_theme = crate::ui::theme::theme();

        self.reset_search(window, cx);
        self.input_state.update(cx, |input, cx| {
            ThemeModeHandler::restore_input(input, window, cx);
        });
        cx.notify();
    }

    /// Render clipboard preview panel.
    fn render_clipboard_preview(
        &self,
        item: Option<&crate::clipboard::ClipboardItem>,
    ) -> impl IntoElement {
        crate::ui::views::clipboard_rendering::render_preview_panel(item)
    }

    /// Simplified navigation - delegates handle their own logic.
    fn select_next(&mut self, _: &SelectNext, window: &mut Window, cx: &mut Context<Self>) {
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

    fn select_prev(&mut self, _: &SelectPrev, window: &mut Window, cx: &mut Context<Self>) {
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
    fn select_tab(&mut self, _: &SelectTab, window: &mut Window, cx: &mut Context<Self>) {
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
    fn select_tab_prev(&mut self, _: &SelectTabPrev, window: &mut Window, cx: &mut Context<Self>) {
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

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
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
                                self.enter_emoji_mode(window, cx);
                                return;
                            }
                            "submenu-clipboard" => {
                                self.enter_clipboard_mode(window, cx);
                                return;
                            }
                            "submenu-themes" => {
                                self.enter_theme_mode(window, cx);
                                return;
                            }
                            _ => {}
                        },
                        ListItem::Ai(_) => {
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
                // No confirmation in AI response mode
            }
        }
    }

    fn cancel(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
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

    fn go_back(&mut self, _: &GoBack, window: &mut Window, cx: &mut Context<Self>) {
        match self.view_mode {
            ViewMode::Main => {
                // Already at main, do nothing
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
}

impl Focusable for LauncherView {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::Render for LauncherView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = &self.current_theme;
        let config = crate::config::config();

        // Input prefix (search icon or back button)
        let input_prefix = match self.view_mode {
            ViewMode::Main => Icon::new(IconName::Search)
                .text_color(cx.theme().muted_foreground)
                .mr_2()
                .into_any_element(),
            ViewMode::EmojiPicker => div()
                .id("back-emoji")
                .cursor_pointer()
                .mr_2()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.exit_emoji_mode(window, cx);
                }))
                .child(Icon::new(IconName::ArrowLeft).text_color(cx.theme().muted_foreground))
                .into_any_element(),
            ViewMode::ClipboardHistory => div()
                .id("back-clipboard")
                .cursor_pointer()
                .mr_2()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.exit_clipboard_mode(window, cx);
                }))
                .child(Icon::new(IconName::ArrowLeft).text_color(cx.theme().muted_foreground))
                .into_any_element(),
            ViewMode::ThemePicker => div()
                .id("back-theme")
                .cursor_pointer()
                .mr_2()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.exit_theme_mode(window, cx);
                }))
                .child(Icon::new(IconName::ArrowLeft).text_color(cx.theme().muted_foreground))
                .into_any_element(),
            ViewMode::AiResponse => div()
                .id("back-ai")
                .cursor_pointer()
                .mr_2()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.exit_ai_mode(window, cx);
                }))
                .child(Icon::new(IconName::ArrowLeft).text_color(cx.theme().muted_foreground))
                .into_any_element(),
        };

        // List content based on mode
        let list_content = match self.view_mode {
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
        };

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
impl AiModeAccess for LauncherView {
    fn ai_mode_handler_mut(&mut self) -> Option<&mut AiModeHandler> {
        self.ai_mode_handler.as_mut()
    }
}
