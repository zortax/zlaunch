mod action;
mod ai;
mod application;
mod calculator;
mod search;
mod submenu;
mod theme;
mod traits;
mod window;

pub use action::{ActionItem, ActionKind};
pub use ai::AiItem;
pub use application::ApplicationItem;
pub use calculator::CalculatorItem;
pub use search::SearchItem;
pub use submenu::{SubmenuItem, SubmenuLayout};
pub use theme::{ThemeItem, ThemeSource};
pub use traits::{Categorizable, DisplayItem, Executable, IconProvider, Previewable};
pub use window::WindowItem;

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
    /// A theme item
    Theme(ThemeItem),
}

impl ListItem {
    /// Get the unique identifier for this item.
    pub fn id(&self) -> &str {
        match self {
            Self::Application(item) => item.id(),
            Self::Window(item) => item.id(),
            Self::Action(item) => item.id(),
            Self::Submenu(item) => item.id(),
            Self::Calculator(item) => item.id(),
            Self::Search(item) => item.id(),
            Self::Ai(item) => item.id(),
            Self::Theme(item) => item.id(),
        }
    }

    /// Get the display name for this item.
    pub fn name(&self) -> &str {
        match self {
            Self::Application(item) => item.name(),
            Self::Window(item) => item.name(),
            Self::Action(item) => item.name(),
            Self::Submenu(item) => item.name(),
            Self::Calculator(item) => item.name(),
            Self::Search(item) => item.name(),
            Self::Ai(item) => item.name(),
            Self::Theme(item) => item.name(),
        }
    }

    /// Get the description/subtitle for this item.
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::Application(item) => item.description(),
            Self::Window(item) => item.description(),
            Self::Action(item) => item.description(),
            Self::Submenu(item) => item.description(),
            Self::Calculator(item) => item.description(),
            Self::Search(item) => item.description(),
            Self::Ai(item) => item.description(),
            Self::Theme(item) => item.description(),
        }
    }

    /// Get the icon path for this item.
    pub fn icon_path(&self) -> Option<&PathBuf> {
        match self {
            Self::Application(item) => item.icon_path(),
            Self::Window(item) => item.icon_path(),
            Self::Action(item) => item.icon_path(),
            Self::Submenu(item) => item.icon_path(),
            Self::Calculator(item) => item.icon_path(),
            Self::Search(item) => item.icon_path(),
            Self::Ai(item) => item.icon_path(),
            Self::Theme(item) => item.icon_path(),
        }
    }

    /// Get the icon name for this item.
    pub fn icon_name(&self) -> Option<&str> {
        match self {
            Self::Application(item) => item.icon_name(),
            Self::Window(item) => item.icon_name(),
            Self::Action(item) => item.icon_name(),
            Self::Submenu(item) => item.icon_name(),
            Self::Calculator(item) => item.icon_name(),
            Self::Search(item) => item.icon_name(),
            Self::Ai(item) => item.icon_name(),
            Self::Theme(item) => item.icon_name(),
        }
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
        match self {
            Self::Application(item) => item.action_label(),
            Self::Window(item) => item.action_label(),
            Self::Action(item) => item.action_label(),
            Self::Submenu(item) => item.action_label(),
            Self::Calculator(item) => item.action_label(),
            Self::Search(item) => item.action_label(),
            Self::Ai(item) => item.action_label(),
            Self::Theme(item) => item.action_label(),
        }
    }

    /// Get the sort priority for this item type.
    /// Lower values appear first in the list.
    pub fn sort_priority(&self) -> u8 {
        match self {
            Self::Application(item) => item.sort_priority(),
            Self::Window(item) => item.sort_priority(),
            Self::Action(item) => item.sort_priority(),
            Self::Submenu(item) => item.sort_priority(),
            Self::Calculator(item) => item.sort_priority(),
            Self::Search(item) => item.sort_priority(),
            Self::Ai(item) => item.sort_priority(),
            Self::Theme(item) => item.sort_priority(),
        }
    }

    /// Get the section name for this item type.
    pub fn section_name(&self) -> &'static str {
        match self {
            Self::Application(item) => item.section_name(),
            Self::Window(item) => item.section_name(),
            Self::Action(item) => item.section_name(),
            Self::Submenu(item) => item.section_name(),
            Self::Calculator(item) => item.section_name(),
            Self::Search(item) => item.section_name(),
            Self::Ai(item) => item.section_name(),
            Self::Theme(item) => item.section_name(),
        }
    }
}

// Convenient From implementations

impl From<ApplicationItem> for ListItem {
    fn from(item: ApplicationItem) -> Self {
        Self::Application(item)
    }
}

impl From<WindowItem> for ListItem {
    fn from(item: WindowItem) -> Self {
        Self::Window(item)
    }
}

impl From<ActionItem> for ListItem {
    fn from(item: ActionItem) -> Self {
        Self::Action(item)
    }
}

impl From<SubmenuItem> for ListItem {
    fn from(item: SubmenuItem) -> Self {
        Self::Submenu(item)
    }
}

impl From<CalculatorItem> for ListItem {
    fn from(item: CalculatorItem) -> Self {
        Self::Calculator(item)
    }
}

impl From<SearchItem> for ListItem {
    fn from(item: SearchItem) -> Self {
        Self::Search(item)
    }
}

impl From<AiItem> for ListItem {
    fn from(item: AiItem) -> Self {
        Self::Ai(item)
    }
}

impl From<ThemeItem> for ListItem {
    fn from(item: ThemeItem) -> Self {
        Self::Theme(item)
    }
}
