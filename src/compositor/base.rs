//! Common functionality shared across compositor implementations.

use super::WindowInfo;

/// Describes the capabilities of a compositor implementation.
#[derive(Debug, Clone, Default)]
pub struct CompositorCapabilities {
    /// Whether the compositor supports blur effects via layer rules.
    pub blur_support: bool,
    /// Whether the compositor supports layer shell protocol.
    pub layer_shell: bool,
    /// Whether window switching is functional.
    pub window_switching: bool,
    /// Whether accurate workspace info is available.
    pub workspace_info: bool,
    /// Whether focus state tracking is accurate.
    pub focus_tracking: bool,
}

impl CompositorCapabilities {
    /// Create capabilities for a fully-featured compositor (Hyprland, Niri).
    pub fn full() -> Self {
        Self {
            blur_support: true,
            layer_shell: true,
            window_switching: true,
            workspace_info: true,
            focus_tracking: true,
        }
    }

    /// Create capabilities for a compositor with limited features (KWin).
    pub fn limited() -> Self {
        Self {
            blur_support: false,
            layer_shell: true,
            window_switching: true,
            workspace_info: false,
            focus_tracking: false,
        }
    }

    /// Create capabilities for the no-op compositor.
    pub fn none() -> Self {
        Self::default()
    }
}

/// Get the display title for a window, falling back to class if title is empty.
///
/// Both Hyprland and Niri use this pattern: if a window has no title,
/// show the application class instead.
pub fn get_display_title(title: &str, class: &str) -> String {
    if title.is_empty() {
        class.to_string()
    } else {
        title.to_string()
    }
}

/// Check if a window class represents the launcher itself.
///
/// Used to filter out zlaunch from the window list to prevent
/// users from switching to the launcher window.
pub fn is_launcher_window(class: &str) -> bool {
    class.to_lowercase() == "zlaunch"
}

/// Filter a list of windows to exclude the launcher window.
pub fn filter_launcher_windows(windows: Vec<WindowInfo>) -> Vec<WindowInfo> {
    windows
        .into_iter()
        .filter(|w| !is_launcher_window(&w.class))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_display_title() {
        assert_eq!(get_display_title("Firefox", "firefox"), "Firefox");
        assert_eq!(get_display_title("", "firefox"), "firefox");
        assert_eq!(get_display_title("", ""), "");
    }

    #[test]
    fn test_is_launcher_window() {
        assert!(is_launcher_window("zlaunch"));
        assert!(is_launcher_window("Zlaunch"));
        assert!(is_launcher_window("ZLAUNCH"));
        assert!(!is_launcher_window("firefox"));
        assert!(!is_launcher_window(""));
    }

    #[test]
    fn test_filter_launcher_windows() {
        let windows = vec![
            WindowInfo {
                address: "1".to_string(),
                title: "Firefox".to_string(),
                class: "firefox".to_string(),
                workspace: 1,
                focused: false,
            },
            WindowInfo {
                address: "2".to_string(),
                title: "Launcher".to_string(),
                class: "zlaunch".to_string(),
                workspace: 1,
                focused: true,
            },
        ];

        let filtered = filter_launcher_windows(windows);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].class, "firefox");
    }
}
