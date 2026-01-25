//! Mode-specific handlers for the launcher.
//!
//! Each mode (AI, Emoji, Clipboard, Theme) has its own handler that encapsulates
//! the mode-specific logic, state, and UI coordination. This keeps the main
//! launcher clean and focused on routing/coordination.
//!
//! # Architecture
//!
//! Mode handlers follow a common pattern but aren't unified by a trait because
//! AiModeHandler is fundamentally different (uses streaming responses instead of ListState).
//!
//! Common patterns across handlers:
//! - `new()` constructor that sets up the mode
//! - `list_state()` or `view()` accessor for the mode's state
//! - Static `setup_input()` and `restore_input()` methods for input field configuration
//!
//! Shared utilities are provided in the `base` module to reduce duplication.

pub mod ai_mode;
pub mod base;
pub mod clipboard_mode;
pub mod emoji_mode;
pub mod theme_mode;

pub use ai_mode::{AiModeAccess, AiModeHandler};
pub use base::{DEFAULT_PLACEHOLDER, clear_input_value, restore_main_input, setup_list_mode_input};
pub use clipboard_mode::ClipboardModeHandler;
pub use emoji_mode::EmojiModeHandler;
pub use theme_mode::ThemeModeHandler;
