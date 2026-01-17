//! List delegates for GPUI list components.
//!
//! Delegates are the data providers and renderers for list views in GPUI.
//! They implement the `ListDelegate` trait from `gpui-component` and handle:
//!
//! - Item rendering
//! - Item filtering (fuzzy search)
//! - Selection management
//! - Section grouping
//!
//! # Delegate Types
//!
//! - [`ItemListDelegate`] - Main launcher list (applications, windows, actions, etc.)
//! - [`EmojiGridDelegate`] - Grid-based emoji picker
//! - [`ClipboardListDelegate`] - Clipboard history with preview panel
//! - [`ThemeListDelegate`] - Theme selection list
//!
//! # Architecture
//!
//! All delegates share common functionality through [`BaseDelegate`], which provides:
//! - Selection state management
//! - Basic navigation (next/prev item)
//! - Consistent selection behavior
//!
//! Filtering is handled by [`item_filter::ItemFilter`] which provides fuzzy matching
//! and respects module ordering from configuration.
//!
//! Section grouping (e.g., "Applications", "Windows") is managed by
//! [`section_manager::SectionManager`] for the main list delegate.

mod base;
mod clipboard_delegate;
mod dynamic_items;
mod emoji_delegate;
mod item_delegate;
mod item_filter;
mod section_manager;
mod theme_delegate;

pub use base::BaseDelegate;
pub use clipboard_delegate::ClipboardListDelegate;
pub use emoji_delegate::EmojiGridDelegate;
pub use item_delegate::ItemListDelegate;
pub use theme_delegate::ThemeListDelegate;
