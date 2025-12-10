//! Mode-specific handlers for the launcher.
//!
//! Each mode (AI, Emoji, Clipboard) has its own handler that encapsulates
//! the mode-specific logic, state, and UI coordination. This keeps the main
//! launcher clean and focused on routing/coordination.

pub mod ai_mode;
pub mod clipboard_mode;
pub mod emoji_mode;
pub mod theme_mode;

pub use ai_mode::{AiModeAccess, AiModeHandler};
pub use clipboard_mode::ClipboardModeHandler;
pub use emoji_mode::EmojiModeHandler;
pub use theme_mode::ThemeModeHandler;
