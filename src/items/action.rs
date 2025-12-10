use std::process::Command;

use super::traits::{Categorizable, DisplayItem, Executable, IconProvider};

/// The kind of action to perform.
#[derive(Clone, Debug)]
pub enum ActionKind {
    /// Shutdown the system
    Shutdown,
    /// Reboot the system
    Reboot,
    /// Suspend the system
    Suspend,
    /// Lock the screen
    Lock,
    /// Log out of the session
    Logout,
    /// Custom command execution
    Command(String),
}

/// An action item representing a functional command (shutdown, reboot, etc.).
#[derive(Clone, Debug)]
pub struct ActionItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_name: Option<String>,
    pub kind: ActionKind,
}

impl ActionItem {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        icon_name: Option<String>,
        kind: ActionKind,
    ) -> Self {
        Self {
            id,
            name,
            description,
            icon_name,
            kind,
        }
    }

    /// Create a built-in action item for the given kind.
    pub fn builtin(kind: ActionKind) -> Self {
        let (id, name, description, icon_name) = match &kind {
            ActionKind::Shutdown => (
                "action-shutdown",
                "Shutdown",
                "Power off the system",
                "power",
            ),
            ActionKind::Reboot => ("action-reboot", "Reboot", "Restart the system", "reboot"),
            ActionKind::Suspend => ("action-suspend", "Suspend", "Suspend to RAM", "moon"),
            ActionKind::Lock => ("action-lock", "Lock Screen", "Lock the session", "lock"),
            ActionKind::Logout => ("action-logout", "Log Out", "End the session", "sign-out"),
            ActionKind::Command(cmd) => {
                return Self {
                    id: format!("action-cmd-{}", cmd.len()),
                    name: "Custom Command".to_string(),
                    description: Some(cmd.clone()),
                    icon_name: Some("terminal".to_string()),
                    kind,
                };
            }
        };

        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: Some(description.to_string()),
            icon_name: Some(icon_name.to_string()),
            kind,
        }
    }

    /// Get all built-in action items.
    pub fn builtins() -> Vec<Self> {
        vec![
            Self::builtin(ActionKind::Shutdown),
            Self::builtin(ActionKind::Reboot),
            Self::builtin(ActionKind::Suspend),
            Self::builtin(ActionKind::Lock),
            Self::builtin(ActionKind::Logout),
        ]
    }
}

impl DisplayItem for ActionItem {
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
        "Run"
    }
}

impl IconProvider for ActionItem {
    fn icon_name(&self) -> Option<&str> {
        self.icon_name.as_deref()
    }
}

impl Executable for ActionItem {
    fn execute(&self) -> anyhow::Result<()> {
        match &self.kind {
            ActionKind::Shutdown => {
                Command::new("systemctl").arg("poweroff").spawn()?;
            }
            ActionKind::Reboot => {
                Command::new("systemctl").arg("reboot").spawn()?;
            }
            ActionKind::Suspend => {
                Command::new("systemctl").arg("suspend").spawn()?;
            }
            ActionKind::Lock => {
                Command::new("loginctl").arg("lock-session").spawn()?;
            }
            ActionKind::Logout => {
                Command::new("loginctl")
                    .args(["terminate-session", "self"])
                    .spawn()?;
            }
            ActionKind::Command(cmd) => {
                Command::new("sh").args(["-c", cmd]).spawn()?;
            }
        }
        Ok(())
    }
}

impl Categorizable for ActionItem {
    fn section_name(&self) -> &'static str {
        "Commands"
    }

    fn sort_priority(&self) -> u8 {
        3
    }
}
