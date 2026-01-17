//! Test utilities and mock factories.
//!
//! This module provides common testing utilities including mock object factories
//! and test helpers. Only compiled in test builds.

use crate::config::{AppConfig, ConfigModule, ConfigSearchProvider};
use crate::items::{ApplicationItem, ListItem, WindowItem};
use std::path::PathBuf;

/// Create a mock AppConfig with default values.
pub fn mock_config() -> AppConfig {
    AppConfig::default()
}

/// Create a mock AppConfig with specific modules enabled.
pub fn mock_config_with_modules(modules: Vec<ConfigModule>) -> AppConfig {
    AppConfig {
        combined_modules: Some(modules),
        ..AppConfig::default()
    }
}

/// Create a mock ApplicationItem.
pub fn mock_application(name: &str) -> ApplicationItem {
    ApplicationItem::new(
        format!("app-{}", name.to_lowercase().replace(' ', "-")),
        name.to_string(),
        format!("/usr/bin/{}", name.to_lowercase().replace(' ', "-")),
        None,
        Some(format!("{} application", name)),
        false,
        PathBuf::from(format!(
            "/usr/share/applications/{}.desktop",
            name.to_lowercase().replace(' ', "-")
        )),
    )
}

/// Create a mock ApplicationItem with description.
pub fn mock_application_with_desc(name: &str, description: &str) -> ApplicationItem {
    ApplicationItem::new(
        format!("app-{}", name.to_lowercase().replace(' ', "-")),
        name.to_string(),
        format!("/usr/bin/{}", name.to_lowercase().replace(' ', "-")),
        None,
        Some(description.to_string()),
        false,
        PathBuf::from(format!(
            "/usr/share/applications/{}.desktop",
            name.to_lowercase().replace(' ', "-")
        )),
    )
}

/// Create a mock WindowItem.
pub fn mock_window(title: &str, app_id: &str) -> WindowItem {
    WindowItem::new(
        format!("window-{}", title.to_lowercase().replace(' ', "-")),
        "0x12345".to_string(),
        title.to_string(),
        app_id.to_string(),
        app_id.to_string(), // app_name same as app_id for simplicity
        None,
        1, // workspace
        false,
    )
}

/// Create a set of mock ApplicationItems for testing.
pub fn mock_applications() -> Vec<ApplicationItem> {
    vec![
        mock_application("Firefox"),
        mock_application("Chrome"),
        mock_application("Code"),
        mock_application("Terminal"),
        mock_application("Files"),
    ]
}

/// Create a set of mock ListItems for testing.
pub fn mock_list_items() -> Vec<ListItem> {
    mock_applications()
        .into_iter()
        .map(ListItem::Application)
        .collect()
}

/// Create a mock SearchProvider configuration.
pub fn mock_search_provider(name: &str, trigger: &str) -> ConfigSearchProvider {
    ConfigSearchProvider {
        name: name.to_string(),
        trigger: trigger.to_string(),
        url: format!("https://{}.example.com/search?q={{query}}", name.to_lowercase()),
        icon: "magnifying-glass".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_application() {
        let app = mock_application("Test App");
        assert_eq!(app.name, "Test App");
        assert_eq!(app.id, "app-test-app");
        assert!(app.description.is_some());
    }

    #[test]
    fn test_mock_applications() {
        let apps = mock_applications();
        assert_eq!(apps.len(), 5);
        assert!(apps.iter().any(|a| a.name == "Firefox"));
    }

    #[test]
    fn test_mock_list_items() {
        let items = mock_list_items();
        assert_eq!(items.len(), 5);
        assert!(matches!(items[0], ListItem::Application(_)));
    }

    #[test]
    fn test_mock_window() {
        let window = mock_window("My Document", "code");
        assert_eq!(window.title, "My Document");
        assert_eq!(window.app_id, "code");
    }

    #[test]
    fn test_mock_config() {
        let config = mock_config();
        assert_eq!(config.window_width, 600.0);
        assert_eq!(config.window_height, 400.0);
    }

    #[test]
    fn test_mock_config_with_modules() {
        let config = mock_config_with_modules(vec![
            ConfigModule::Applications,
            ConfigModule::Calculator,
        ]);
        assert_eq!(config.combined_modules.unwrap().len(), 2);
    }

    #[test]
    fn test_mock_search_provider() {
        let provider = mock_search_provider("Google", "!g");
        assert_eq!(provider.name, "Google");
        assert_eq!(provider.trigger, "!g");
        assert!(provider.url.contains("{query}"));
    }
}
