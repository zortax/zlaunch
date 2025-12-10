use crate::desktop::DesktopEntry;
use std::path::PathBuf;

use super::traits::{Categorizable, DisplayItem, Executable, IconProvider};

/// An application item representing a desktop application.
#[derive(Clone, Debug)]
pub struct ApplicationItem {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon_path: Option<PathBuf>,
    pub description: Option<String>,
    pub terminal: bool,
    pub desktop_path: PathBuf,
}

impl ApplicationItem {
    pub fn new(
        id: String,
        name: String,
        exec: String,
        icon_path: Option<PathBuf>,
        description: Option<String>,
        terminal: bool,
        desktop_path: PathBuf,
    ) -> Self {
        Self {
            id,
            name,
            exec,
            icon_path,
            description,
            terminal,
            desktop_path,
        }
    }
}

impl From<DesktopEntry> for ApplicationItem {
    fn from(entry: DesktopEntry) -> Self {
        Self {
            id: entry.id,
            name: entry.name,
            exec: entry.exec,
            icon_path: entry.icon_path,
            description: entry.comment,
            terminal: entry.terminal,
            desktop_path: entry.path,
        }
    }
}

impl From<&DesktopEntry> for ApplicationItem {
    fn from(entry: &DesktopEntry) -> Self {
        Self {
            id: entry.id.clone(),
            name: entry.name.clone(),
            exec: entry.exec.clone(),
            icon_path: entry.icon_path.clone(),
            description: entry.comment.clone(),
            terminal: entry.terminal,
            desktop_path: entry.path.clone(),
        }
    }
}

impl DisplayItem for ApplicationItem {
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

impl IconProvider for ApplicationItem {
    fn icon_path(&self) -> Option<&PathBuf> {
        self.icon_path.as_ref()
    }
}

impl Executable for ApplicationItem {
    fn execute(&self) -> anyhow::Result<()> {
        // Execution is handled at a higher level with DesktopEntry
        // This is just a placeholder for the trait
        Ok(())
    }
}

impl Categorizable for ApplicationItem {
    fn section_name(&self) -> &'static str {
        "Applications"
    }

    fn sort_priority(&self) -> u8 {
        4
    }
}
