//! Launcher item types and traits.
//!
//! This module defines all the item types that can appear in the launcher,
//! along with the traits that describe their behavior.
//!
//! # Item Types
//!
//! Each item type represents a different kind of launcher entry:
//!
//! - [`ApplicationItem`] - Desktop applications (from .desktop files)
//! - [`WindowItem`] - Open windows for window switching
//! - [`ActionItem`] - System actions (shutdown, reboot, logout)
//! - [`CalculatorItem`] - Mathematical calculation results
//! - [`SearchItem`] - Web search queries
//! - [`AiItem`] - AI/LLM query interface
//! - [`ThemeItem`] - Theme selection entries
//! - [`SubmenuItem`] - Nested submenus
//!
//! # The ListItem Enum
//!
//! All item types are unified under the [`ListItem`] enum, which allows the
//! launcher to work with heterogeneous collections of items. The enum uses
//! a dispatch macro system ([`dispatch_item!`]) to delegate trait method calls
//! to the appropriate variant.
//!
//! # Core Traits
//!
//! Items implement several traits from the [`traits`] module:
//!
//! - [`DisplayItem`] - Required: id, name, description, action_label
//! - [`IconProvider`] - Icon path or Phosphor icon name
//! - [`Executable`] - How to execute/activate the item
//! - [`Categorizable`] - Section grouping and sort priority
//! - [`Previewable`] - Preview content for preview panels
//!
//! # Design Decisions
//!
//! The dispatch macro approach was chosen over trait objects because:
//! 1. All item types are known at compile time
//! 2. It avoids boxing overhead for common operations
//! 3. The enum can be cloned and compared easily

mod action;
mod ai;
mod application;
mod calculator;
mod dispatch;
mod search;
mod submenu;
mod theme;
mod traits;
mod window;

use dispatch::dispatch_item;

pub use action::{ActionItem, ActionKind};
pub use ai::AiItem;
pub use application::ApplicationItem;
pub use calculator::CalculatorItem;
pub use search::SearchItem;
pub use submenu::{SubmenuItem, SubmenuLayout};
pub use theme::{ThemeItem, ThemeSource};
pub use traits::{Categorizable, DisplayItem, Executable, IconProvider, Previewable};
pub use window::WindowItem;

use crate::config::ConfigModule;
use std::path::PathBuf;

/// A list item that can be displayed in the launcher.
/// This enum abstracts over different types of items that can appear in the list.
#[derive(Clone, Debug)]
pub enum ListItem {
    /// A desktop application
    Application(ApplicationItem),
    /// An open window (for window switching)
    Window(WindowItem),
    /// A functional action (shutdown, reboot, etc.)
    Action(ActionItem),
    /// A submenu that opens a nested view
    Submenu(SubmenuItem),
    /// A calculator result
    Calculator(CalculatorItem),
    /// A web search item
    Search(SearchItem),
    /// An AI query item
    Ai(AiItem),
    /// A theme item (boxed due to large size)
    Theme(Box<ThemeItem>),
}

impl ListItem {
    /// Get the unique identifier for this item.
    pub fn id(&self) -> &str {
        dispatch_item!(self, id)
    }

    /// Get the display name for this item.
    pub fn name(&self) -> &str {
        dispatch_item!(self, name)
    }

    /// Get the description/subtitle for this item.
    pub fn description(&self) -> Option<&str> {
        dispatch_item!(self, description)
    }

    /// Get the icon path for this item.
    pub fn icon_path(&self) -> Option<&PathBuf> {
        dispatch_item!(self, icon_path)
    }

    /// Get the icon name for this item.
    pub fn icon_name(&self) -> Option<&str> {
        dispatch_item!(self, icon_name)
    }

    /// Check if this item is a submenu.
    pub fn is_submenu(&self) -> bool {
        matches!(self, Self::Submenu(_))
    }

    /// Check if this item is an application.
    pub fn is_application(&self) -> bool {
        matches!(self, Self::Application(_))
    }

    /// Check if this item is a window.
    pub fn is_window(&self) -> bool {
        matches!(self, Self::Window(_))
    }

    /// Check if this item is an action.
    pub fn is_action(&self) -> bool {
        matches!(self, Self::Action(_))
    }

    /// Check if this item is a calculator result.
    pub fn is_calculator(&self) -> bool {
        matches!(self, Self::Calculator(_))
    }

    /// Get the action label to display (e.g., "Open", "Switch", "Run").
    pub fn action_label(&self) -> &'static str {
        dispatch_item!(self, action_label)
    }

    /// Get the sort priority for this item type.
    /// Lower values appear first in the list.
    pub fn sort_priority(&self) -> u8 {
        dispatch_item!(self, sort_priority)
    }

    /// Get the section name for this item type.
    pub fn section_name(&self) -> &'static str {
        dispatch_item!(self, section_name)
    }

    /// Get the ConfigModule this item belongs to.
    /// This method has custom logic per variant and cannot use dispatch_item!.
    pub fn config_module(&self) -> ConfigModule {
        match self {
            Self::Application(_) => ConfigModule::Applications,
            Self::Window(_) => ConfigModule::Windows,
            Self::Action(_) => ConfigModule::Actions,
            Self::Submenu(item) => {
                // Map submenu IDs to their modules
                match item.id.as_str() {
                    "submenu-emojis" => ConfigModule::Emojis,
                    "submenu-clipboard" => ConfigModule::Clipboard,
                    "submenu-themes" => ConfigModule::Themes,
                    _ => ConfigModule::Actions, // Default fallback
                }
            }
            Self::Calculator(_) => ConfigModule::Calculator,
            Self::Search(_) => ConfigModule::Search,
            Self::Ai(_) => ConfigModule::Ai,
            Self::Theme(_) => ConfigModule::Themes,
        }
    }
}
