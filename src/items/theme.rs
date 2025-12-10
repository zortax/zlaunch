use super::traits::{Categorizable, DisplayItem, IconProvider};
use crate::ui::theme::LauncherTheme;

/// The source of a theme.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeSource {
    /// A bundled theme (shipped with zlaunch)
    Bundled,
    /// A user-defined theme from ~/.config/zlaunch/themes/
    UserDefined(String),
}

impl ThemeSource {
    /// Get a description string for display purposes.
    pub fn description(&self) -> String {
        match self {
            Self::Bundled => "bundled".to_string(),
            Self::UserDefined(filename) => filename.clone(),
        }
    }
}

/// A theme item that can be selected in the theme picker.
#[derive(Clone, Debug)]
pub struct ThemeItem {
    pub name: String,
    pub source: ThemeSource,
    pub theme: LauncherTheme,
    pub description: String,
}

impl ThemeItem {
    /// Create a new theme item.
    pub fn new(name: String, source: ThemeSource, theme: LauncherTheme) -> Self {
        let description = source.description();
        Self {
            name,
            source,
            theme,
            description,
        }
    }

    /// Create a bundled theme item.
    pub fn bundled(name: String, theme: LauncherTheme) -> Self {
        Self::new(name, ThemeSource::Bundled, theme)
    }

    /// Create a user-defined theme item.
    pub fn user_defined(name: String, filename: String, theme: LauncherTheme) -> Self {
        Self::new(name, ThemeSource::UserDefined(filename), theme)
    }
}

impl DisplayItem for ThemeItem {
    fn id(&self) -> &str {
        &self.name
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        Some(&self.description)
    }

    fn action_label(&self) -> &'static str {
        "Apply"
    }
}

impl IconProvider for ThemeItem {
    fn icon_name(&self) -> Option<&str> {
        // We'll render a custom dynamic icon, so no icon name needed
        None
    }
}

impl Categorizable for ThemeItem {
    fn section_name(&self) -> &'static str {
        "Themes"
    }

    fn sort_priority(&self) -> u8 {
        // Themes should appear in their own section
        10
    }
}
