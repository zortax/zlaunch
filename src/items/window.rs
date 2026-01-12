use super::traits::{Categorizable, DisplayItem, Executable, IconProvider};
use crate::compositor::WindowInfo;
use std::path::PathBuf;

/// A window item representing an open window for window switching.
#[derive(Clone, Debug)]
pub struct WindowItem {
    /// Internal ID for the list
    pub id: String,
    /// Compositor-specific window address (used for focusing)
    pub address: String,
    /// Window title
    pub title: String,
    /// Application class/ID (e.g., "firefox")
    pub app_id: String,
    /// Human-readable application name
    pub app_name: String,
    /// Pre-computed description (e.g., "Firefox - Workspace 2")
    pub description: String,
    /// Resolved icon path
    pub icon_path: Option<PathBuf>,
    /// Workspace number
    pub workspace: i32,
    /// Whether this window is currently focused
    pub focused: bool,
}

impl WindowItem {
    /// Create a new window item directly with all fields.
    ///
    /// Prefer using `from_window_info` when creating from compositor data.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        address: String,
        title: String,
        app_id: String,
        app_name: String,
        icon_path: Option<PathBuf>,
        workspace: i32,
        focused: bool,
    ) -> Self {
        let description = format!("{} - Workspace {}", app_name, workspace);
        Self {
            id,
            address,
            title,
            app_id,
            app_name,
            description,
            icon_path,
            workspace,
            focused,
        }
    }

    /// Create a WindowItem from compositor WindowInfo.
    pub fn from_window_info(info: WindowInfo, icon_path: Option<PathBuf>) -> Self {
        let app_name = titlecase_app_name(&info.class);
        let description = format!("{} - Workspace {}", app_name, info.workspace);
        Self {
            id: format!("window-{}", info.address),
            address: info.address,
            title: info.title,
            app_id: info.class.clone(),
            app_name,
            description,
            icon_path,
            workspace: info.workspace,
            focused: info.focused,
        }
    }
}

impl DisplayItem for WindowItem {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.title
    }

    fn description(&self) -> Option<&str> {
        Some(&self.description)
    }

    fn action_label(&self) -> &'static str {
        "Switch"
    }
}

impl IconProvider for WindowItem {
    fn icon_path(&self) -> Option<&PathBuf> {
        self.icon_path.as_ref()
    }
}

impl Executable for WindowItem {
    fn execute(&self) -> anyhow::Result<()> {
        // Note: This will need access to compositor
        // We'll handle this through a callback mechanism in the UI layer
        Ok(())
    }
}

impl Categorizable for WindowItem {
    fn section_name(&self) -> &'static str {
        "Windows"
    }

    fn sort_priority(&self) -> u8 {
        2
    }
}

/// Convert an app class to a human-readable name.
fn titlecase_app_name(class: &str) -> String {
    let name = class.rsplit('.').next().unwrap_or(class);
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}
