//! Launcher view - the main UI component of zlaunch.
//!
//! This module contains [`LauncherView`], the primary GPUI component that renders
//! the launcher window and handles all user interaction.
//!
//! # Architecture
//!
//! The launcher is split into several submodules for maintainability:
//!
//! - [`state`] - View state management ([`ViewMode`], [`ModeState`])
//! - [`actions`] - Action handlers for keyboard/mouse events
//! - [`mode_switching`] - Logic for switching between launcher modes
//! - [`navigation`] - Item selection and list navigation
//! - [`render`] - UI rendering implementation
//!
//! # View Modes
//!
//! The launcher supports multiple view modes:
//!
//! - **Main** - Combined view showing applications, windows, calculator, etc.
//! - **EmojiPicker** - Grid-based emoji selection
//! - **ClipboardHistory** - List of recent clipboard entries with preview
//! - **AiResponse** - Streaming AI chat interface
//! - **ThemePicker** - Theme selection with live preview
//! - **Combined** - Customizable combined view with module ordering
//!
//! # Key Bindings
//!
//! - `Up/Down` - Navigate items
//! - `Tab/Shift+Tab` - Grid navigation (emoji mode)
//! - `Ctrl+Tab/Ctrl+Shift+Tab` - Switch between modes
//! - `Enter` - Execute selected item
//! - `Escape` - Hide launcher or go back
//! - `Backspace` (empty input) - Return to previous mode

mod actions;
mod mode_switching;
mod navigation;
mod render;
mod state;

pub use state::{ModeState, ViewMode};

use std::sync::Arc;

use gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, KeyBinding, Window, actions};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::list::ListState;

use crate::compositor::Compositor;
use crate::config::{ConfigModule, LauncherMode, get_combined_modules};
use crate::items::ListItem;
use crate::ui::delegates::ItemListDelegate;
use crate::ui::modes::{
    AiModeAccess, AiModeHandler, ClipboardModeHandler, EmojiModeHandler, ThemeModeHandler,
};
use crate::ui::theme::LauncherTheme;

// Action definitions
actions!(
    launcher,
    [
        SelectNext,
        SelectPrev,
        SelectTab,
        SelectTabPrev,
        Confirm,
        Cancel,
        GoBack,
        SwitchModeNext,
        SwitchModePrev
    ]
);

/// Initialize key bindings for the launcher view.
pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", SelectPrev, Some("LauncherView")),
        KeyBinding::new("down", SelectNext, Some("LauncherView")),
        KeyBinding::new("tab", SelectTab, Some("LauncherView")),
        KeyBinding::new("shift-tab", SelectTabPrev, Some("LauncherView")),
        KeyBinding::new("enter", Confirm, Some("LauncherView")),
        KeyBinding::new("escape", Cancel, Some("LauncherView")),
        KeyBinding::new("backspace", GoBack, Some("LauncherView")),
        KeyBinding::new("ctrl-tab", SwitchModeNext, Some("LauncherView")),
        KeyBinding::new("ctrl-shift-tab", SwitchModePrev, Some("LauncherView")),
    ]);
}

/// The main launcher view.
pub struct LauncherView {
    /// Current view mode
    pub(crate) view_mode: ViewMode,
    /// Mode state (tracks active modes and current mode index)
    pub(crate) mode_state: ModeState,
    /// Whether we navigated into a submenu from combined view (vs direct mode)
    pub(crate) navigated_into_submenu: bool,
    /// Main list state
    pub(crate) list_state: Entity<ListState<ItemListDelegate>>,
    /// Original items (for recreating filtered delegates)
    pub(crate) original_items: Vec<ListItem>,
    /// Compositor reference (for item confirm callbacks)
    pub(crate) compositor: Arc<dyn Compositor>,
    /// Emoji mode handler (created on demand)
    pub(crate) emoji_mode_handler: Option<EmojiModeHandler>,
    /// Clipboard mode handler (created on demand)
    pub(crate) clipboard_mode_handler: Option<ClipboardModeHandler>,
    /// AI mode handler (created on demand)
    pub(crate) ai_mode_handler: Option<AiModeHandler>,
    /// Theme mode handler (created on demand)
    pub(crate) theme_mode_handler: Option<ThemeModeHandler>,
    /// Current theme (for live preview)
    pub(crate) current_theme: LauncherTheme,
    /// Theme preview subscription
    pub(crate) _theme_preview_subscription: Option<gpui::Subscription>,
    /// Input state
    pub(crate) input_state: Entity<InputState>,
    /// Focus handle
    pub(crate) focus_handle: FocusHandle,
    /// Callback to hide the launcher
    pub(crate) on_hide: Arc<dyn Fn() + Send + Sync>,
}

impl LauncherView {
    /// Create a new launcher view with specified modes.
    pub fn new(
        items: Vec<ListItem>,
        compositor: Arc<dyn Compositor>,
        modes: Vec<LauncherMode>,
        on_hide: impl Fn() + Send + Sync + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let on_hide = Arc::new(on_hide);
        let mode_state = ModeState::new(modes);

        // Determine modules to show based on current mode
        let modules_for_delegate = Self::modules_for_mode(mode_state.current_mode());

        // Create main delegate with callbacks
        let mut delegate = ItemListDelegate::new(items.clone(), modules_for_delegate);
        let on_hide_for_confirm = on_hide.clone();
        let compositor_for_confirm = compositor.clone();

        delegate.set_on_confirm(move |item| {
            Self::handle_item_confirm(item, &compositor_for_confirm);
            on_hide_for_confirm();
        });

        let on_hide_for_cancel = on_hide.clone();
        delegate.set_on_cancel(move || on_hide_for_cancel());

        let list_state = cx.new(|cx| ListState::new(delegate, window, cx));

        // Create input state with placeholder based on initial mode
        let initial_placeholder = Self::placeholder_for_mode(mode_state.current_mode());
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder(initial_placeholder));

        // Subscribe to input changes
        let list_state_for_subscribe = list_state.clone();
        cx.subscribe(
            &input_state,
            move |_this, input: Entity<InputState>, event: &InputEvent, cx: &mut Context<Self>| {
                if let InputEvent::Change = event {
                    let text = input.read(cx).value().to_string();
                    // Update the delegate's query directly (synchronous filtering)
                    list_state_for_subscribe.update(
                        cx,
                        |state: &mut ListState<ItemListDelegate>,
                         cx: &mut Context<ListState<ItemListDelegate>>| {
                            state.delegate_mut().set_query(text);
                            cx.notify();
                        },
                    );
                }
            },
        )
        .detach();

        let focus_handle = cx.focus_handle();

        // Hide when the view loses focus
        let on_hide_for_blur = on_hide.clone();
        cx.on_blur(&focus_handle, window, move |_this, _window, _cx| {
            on_hide_for_blur();
        })
        .detach();

        // Determine initial view mode based on current launcher mode
        let initial_view_mode = match mode_state.current_mode() {
            LauncherMode::Combined => ViewMode::Main,
            LauncherMode::Emojis => ViewMode::EmojiPicker,
            LauncherMode::Clipboard => ViewMode::ClipboardHistory,
            LauncherMode::Themes => ViewMode::ThemePicker,
            LauncherMode::Ai => ViewMode::AiResponse,
            // For other modes (Applications, Windows, Actions, Search, Calculator),
            // use Main view with filtered delegate
            _ => ViewMode::Main,
        };

        let mut launcher = Self {
            view_mode: initial_view_mode,
            mode_state,
            navigated_into_submenu: false,
            list_state,
            original_items: items,
            compositor,
            emoji_mode_handler: None,
            clipboard_mode_handler: None,
            ai_mode_handler: None,
            theme_mode_handler: None,
            current_theme: crate::config::load_configured_theme(),
            _theme_preview_subscription: None,
            input_state,
            focus_handle,
            on_hide,
        };

        // Initialize mode handler if starting in a direct mode
        launcher.initialize_direct_mode(window, cx);

        launcher
    }

    /// Get the modules to show for a given launcher mode.
    pub fn modules_for_mode(mode: &LauncherMode) -> Vec<ConfigModule> {
        match mode {
            LauncherMode::Combined => get_combined_modules(),
            // Modes with dedicated handlers - return combined modules
            // (they don't use the main delegate anyway)
            LauncherMode::Emojis
            | LauncherMode::Clipboard
            | LauncherMode::Themes
            | LauncherMode::Ai => get_combined_modules(),
            // Single-module modes - return just that module
            LauncherMode::Applications => vec![ConfigModule::Applications],
            LauncherMode::Windows => vec![ConfigModule::Windows],
            LauncherMode::Actions => vec![ConfigModule::Actions],
            LauncherMode::Search => vec![ConfigModule::Search],
            LauncherMode::Calculator => vec![ConfigModule::Calculator],
        }
    }

    /// Get the placeholder text for a given launcher mode.
    pub fn placeholder_for_mode(mode: &LauncherMode) -> &'static str {
        match mode {
            LauncherMode::Combined => "Search anything...",
            LauncherMode::Applications => "Search applications...",
            LauncherMode::Windows => "Search windows...",
            LauncherMode::Actions => "Search actions...",
            LauncherMode::Emojis => "Search emojis...",
            LauncherMode::Clipboard => "Search clipboard...",
            LauncherMode::Themes => "Search themes...",
            LauncherMode::Ai => "Ask AI...",
            LauncherMode::Search => "Search the web...",
            LauncherMode::Calculator => "Calculate...",
        }
    }

    /// Initialize mode handler if starting in a direct mode.
    fn initialize_direct_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.mode_state.current_mode() {
            LauncherMode::Combined => {} // No special initialization
            LauncherMode::Emojis => {
                self.enter_emoji_mode(window, cx);
            }
            LauncherMode::Clipboard => {
                self.enter_clipboard_mode(window, cx);
            }
            LauncherMode::Themes => {
                self.enter_theme_mode(window, cx);
            }
            _ => {} // Other modes use filtered main view
        }
    }

    /// Refresh the current theme from the global state.
    /// Called when the theme is changed via IPC while the window is open.
    pub fn refresh_theme(&mut self, cx: &mut Context<Self>) {
        self.current_theme = crate::ui::theme::theme();
        cx.notify();
    }

    /// Refresh the application list after file changes.
    /// Called when the daemon detects changes to installed applications.
    pub fn refresh_applications(
        &mut self,
        applications: Vec<crate::items::ApplicationItem>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Update original_items with new applications
        self.original_items = applications
            .into_iter()
            .map(ListItem::Application)
            .collect();

        // Recreate the delegate (reuses existing mode_switching.rs logic)
        self.recreate_delegate_for_mode(window, cx);
        cx.notify();
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
        let placeholder = Self::placeholder_for_mode(self.mode_state.current_mode());
        self.input_state.update(cx, |input, cx| {
            input.set_value("", window, cx);
            input.set_placeholder(placeholder, window, cx);
        });
    }
}

impl Focusable for LauncherView {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl AiModeAccess for LauncherView {
    fn ai_mode_handler_mut(&mut self) -> Option<&mut AiModeHandler> {
        self.ai_mode_handler.as_mut()
    }
}
