use std::path::PathBuf;

/// Trait for items that have basic display properties
pub trait DisplayItem {
    /// Get the unique identifier for this item
    fn id(&self) -> &str;

    /// Get the display name/title for this item
    fn name(&self) -> &str;

    /// Get the description/subtitle for this item
    fn description(&self) -> Option<&str>;

    /// Get the action label (e.g., "Open", "Switch", "Run")
    fn action_label(&self) -> &'static str;
}

/// Trait for items that have icons
pub trait IconProvider {
    /// Get the icon path if available
    fn icon_path(&self) -> Option<&PathBuf> {
        None
    }

    /// Get the icon name (for named icons like Phosphor icons)
    fn icon_name(&self) -> Option<&str> {
        None
    }
}

/// Trait for items that can be executed/launched
pub trait Executable {
    /// Execute this item's action
    fn execute(&self) -> anyhow::Result<()>;
}

/// Trait for items that can provide a preview
pub trait Previewable {
    /// Check if this item has a preview
    fn has_preview(&self) -> bool {
        false
    }

    /// Get preview content as a string (if applicable)
    fn preview_content(&self) -> Option<String> {
        None
    }
}

/// Trait for items that belong to sections
pub trait Categorizable {
    /// Get the section name for grouping
    fn section_name(&self) -> &'static str;

    /// Get the sort priority (lower values appear first)
    fn sort_priority(&self) -> u8;
}
