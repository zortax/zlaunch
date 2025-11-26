mod action;
mod application;
mod submenu;
mod window;

pub use action::{ActionItem, ActionKind};
pub use application::ApplicationItem;
pub use submenu::{SubmenuItem, SubmenuLayout};
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
}

impl ListItem {
    /// Get the unique identifier for this item.
    pub fn id(&self) -> &str {
        match self {
            Self::Application(app) => &app.id,
            Self::Window(win) => &win.id,
            Self::Action(act) => &act.id,
            Self::Submenu(sub) => &sub.id,
        }
    }

    /// Get the display name for this item.
    pub fn name(&self) -> &str {
        match self {
            Self::Application(app) => &app.name,
            Self::Window(win) => &win.title,
            Self::Action(act) => &act.name,
            Self::Submenu(sub) => &sub.name,
        }
    }

    /// Get the description/subtitle for this item.
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::Application(app) => app.description.as_deref(),
            Self::Window(win) => Some(&win.description),
            Self::Action(act) => act.description.as_deref(),
            Self::Submenu(sub) => sub.description.as_deref(),
        }
    }

    /// Get the icon path for this item.
    pub fn icon_path(&self) -> Option<&PathBuf> {
        match self {
            Self::Application(app) => app.icon_path.as_ref(),
            Self::Window(win) => win.icon_path.as_ref(),
            Self::Action(_) => None,  // Actions use icon names, not paths
            Self::Submenu(_) => None, // Submenus use icon names, not paths
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

    /// Get the action label to display (e.g., "Open", "Switch", "Run").
    pub fn action_label(&self) -> &'static str {
        match self {
            Self::Application(_) => "Open",
            Self::Window(_) => "Switch",
            Self::Action(_) => "Run",
            Self::Submenu(_) => "Open",
        }
    }

    /// Get the sort priority for this item type.
    /// Lower values appear first in the list.
    /// Windows (0) < Applications (1) < Actions (2) < Submenus (3)
    pub fn sort_priority(&self) -> u8 {
        match self {
            Self::Window(_) => 0,
            Self::Application(_) => 1,
            Self::Action(_) => 2,
            Self::Submenu(_) => 3,
        }
    }

    /// Get the section name for this item type.
    pub fn section_name(&self) -> &'static str {
        match self {
            Self::Window(_) => "Windows",
            Self::Application(_) => "Applications",
            Self::Action(_) => "Actions",
            Self::Submenu(_) => "Submenus",
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
