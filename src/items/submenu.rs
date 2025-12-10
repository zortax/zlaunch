use super::traits::{Categorizable, DisplayItem, IconProvider};

/// The layout style for a submenu.
#[derive(Clone, Debug, Default)]
pub enum SubmenuLayout {
    /// Standard vertical list (default)
    #[default]
    List,
    /// Grid layout (e.g., for emoji picker)
    Grid { columns: usize },
    /// Custom layout identified by name
    Custom(String),
}

/// A submenu item that opens a nested list or custom UI.
#[derive(Clone, Debug)]
pub struct SubmenuItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_name: Option<String>,
    pub layout: SubmenuLayout,
}

impl SubmenuItem {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        icon_name: Option<String>,
        layout: SubmenuLayout,
    ) -> Self {
        Self {
            id,
            name,
            description,
            icon_name,
            layout,
        }
    }

    /// Create a simple list submenu
    pub fn list(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            icon_name: None,
            layout: SubmenuLayout::List,
        }
    }

    /// Create a grid submenu
    pub fn grid(id: impl Into<String>, name: impl Into<String>, columns: usize) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            icon_name: None,
            layout: SubmenuLayout::Grid { columns },
        }
    }

    /// Create a custom layout submenu
    pub fn custom(
        id: impl Into<String>,
        name: impl Into<String>,
        layout_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            icon_name: None,
            layout: SubmenuLayout::Custom(layout_name.into()),
        }
    }

    /// Builder method to set a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Builder method to set an icon name.
    pub fn with_icon(mut self, icon_name: impl Into<String>) -> Self {
        self.icon_name = Some(icon_name.into());
        self
    }
}

impl DisplayItem for SubmenuItem {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn action_label(&self) -> &'static str {
        "Open"
    }
}

impl IconProvider for SubmenuItem {
    fn icon_name(&self) -> Option<&str> {
        self.icon_name.as_deref()
    }
}

impl Categorizable for SubmenuItem {
    fn section_name(&self) -> &'static str {
        "Commands"
    }

    fn sort_priority(&self) -> u8 {
        3
    }
}
