//! Configuration validation utilities.
//!
//! Provides validation for configuration values, returning warnings for
//! non-fatal issues that should be logged but don't prevent startup.

use super::theme_loader::list_themes;
use super::types::{AppConfig, ConfigSearchProvider};

/// Non-fatal validation warning.
#[derive(Debug)]
pub struct ValidationWarning {
    /// The field that has an issue.
    pub field: String,
    /// A description of the issue.
    pub message: String,
}

/// Validate the entire config, returning warnings for non-fatal issues.
///
/// This function checks for:
/// - Window dimensions outside recommended ranges
/// - Search provider URLs missing the `{query}` placeholder
/// - Invalid trigger formats for search providers
pub fn validate_config(config: &AppConfig) -> Vec<ValidationWarning> {
    let mut warnings = vec![];

    // Validate window dimensions
    if config.window_width < 300.0 {
        warnings.push(ValidationWarning {
            field: "window_width".to_string(),
            message: format!(
                "Width {} is below minimum (300). Consider increasing for usability.",
                config.window_width
            ),
        });
    } else if config.window_width > 2000.0 {
        warnings.push(ValidationWarning {
            field: "window_width".to_string(),
            message: format!(
                "Width {} exceeds maximum (2000). This may cause display issues.",
                config.window_width
            ),
        });
    }

    if config.window_height < 200.0 {
        warnings.push(ValidationWarning {
            field: "window_height".to_string(),
            message: format!(
                "Height {} is below minimum (200). Consider increasing for usability.",
                config.window_height
            ),
        });
    } else if config.window_height > 1500.0 {
        warnings.push(ValidationWarning {
            field: "window_height".to_string(),
            message: format!(
                "Height {} exceeds maximum (1500). This may cause display issues.",
                config.window_height
            ),
        });
    }

    // Validate search providers
    if let Some(providers) = &config.search_providers {
        for provider in providers {
            warnings.extend(validate_search_provider(provider));
        }
    }

    // Validate theme exists (only if non-default)
    if !config.theme.is_empty() && config.theme != "default" && !validate_theme_name(&config.theme)
    {
        warnings.push(ValidationWarning {
            field: "theme".to_string(),
            message: format!(
                "Theme '{}' not found. Will fall back to default theme.",
                config.theme
            ),
        });
    }

    warnings
}

/// Validate a search provider configuration.
fn validate_search_provider(provider: &ConfigSearchProvider) -> Vec<ValidationWarning> {
    let mut warnings = vec![];

    // Check URL contains {query} placeholder
    if !provider.url.contains("{query}") {
        warnings.push(ValidationWarning {
            field: format!("search_providers.{}.url", provider.name),
            message: format!(
                "URL for '{}' must contain {{query}} placeholder. Search will not work correctly.",
                provider.name
            ),
        });
    }

    // Check URL looks valid (basic check)
    if !provider.url.starts_with("http://") && !provider.url.starts_with("https://") {
        warnings.push(ValidationWarning {
            field: format!("search_providers.{}.url", provider.name),
            message: format!(
                "URL for '{}' should start with http:// or https://",
                provider.name
            ),
        });
    }

    // Warn if trigger doesn't start with ! or : (common convention)
    if !provider.trigger.is_empty()
        && !provider.trigger.starts_with('!')
        && !provider.trigger.starts_with(':')
    {
        warnings.push(ValidationWarning {
            field: format!("search_providers.{}.trigger", provider.name),
            message: format!(
                "Trigger '{}' for '{}' doesn't start with ! or :. This is allowed but unconventional.",
                provider.trigger, provider.name
            ),
        });
    }

    // Check trigger isn't too long
    if provider.trigger.len() > 10 {
        warnings.push(ValidationWarning {
            field: format!("search_providers.{}.trigger", provider.name),
            message: format!(
                "Trigger '{}' is quite long. Shorter triggers are easier to type.",
                provider.trigger
            ),
        });
    }

    warnings
}

/// Check if a theme name exists.
pub fn validate_theme_name(name: &str) -> bool {
    list_themes().contains(&name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_config() {
        let config = AppConfig::default();
        let warnings = validate_config(&config);
        // Default config should have no warnings
        assert!(warnings.is_empty(), "Warnings: {:?}", warnings);
    }

    #[test]
    fn test_validate_window_width_too_small() {
        let config = AppConfig {
            window_width: 100.0,
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(warnings.iter().any(|w| w.field == "window_width"));
    }

    #[test]
    fn test_validate_window_width_too_large() {
        let config = AppConfig {
            window_width: 3000.0,
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(warnings.iter().any(|w| w.field == "window_width"));
    }

    #[test]
    fn test_validate_window_height_too_small() {
        let config = AppConfig {
            window_height: 50.0,
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(warnings.iter().any(|w| w.field == "window_height"));
    }

    #[test]
    fn test_validate_search_provider_missing_query() {
        let config = AppConfig {
            search_providers: Some(vec![ConfigSearchProvider {
                name: "BadProvider".to_string(),
                trigger: "!bad".to_string(),
                url: "https://example.com/search".to_string(), // Missing {query}
                icon: "magnifying-glass".to_string(),
            }]),
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(
            warnings
                .iter()
                .any(|w| w.field.contains("BadProvider") && w.message.contains("{query}"))
        );
    }

    #[test]
    fn test_validate_search_provider_invalid_url() {
        let config = AppConfig {
            search_providers: Some(vec![ConfigSearchProvider {
                name: "NoProtocol".to_string(),
                trigger: "!np".to_string(),
                url: "example.com/search?q={query}".to_string(), // Missing protocol
                icon: "magnifying-glass".to_string(),
            }]),
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(
            warnings
                .iter()
                .any(|w| w.field.contains("NoProtocol") && w.message.contains("http"))
        );
    }

    #[test]
    fn test_validate_search_provider_unconventional_trigger() {
        let config = AppConfig {
            search_providers: Some(vec![ConfigSearchProvider {
                name: "WeirdTrigger".to_string(),
                trigger: "search".to_string(), // Doesn't start with ! or :
                url: "https://example.com/search?q={query}".to_string(),
                icon: "magnifying-glass".to_string(),
            }]),
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(
            warnings
                .iter()
                .any(|w| w.field.contains("WeirdTrigger") && w.message.contains("unconventional"))
        );
    }

    #[test]
    fn test_validate_nonexistent_theme() {
        let config = AppConfig {
            theme: "nonexistent-theme-xyz".to_string(),
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(warnings.iter().any(|w| w.field == "theme"));
    }

    #[test]
    fn test_validate_default_theme_no_warning() {
        let config = AppConfig {
            theme: "default".to_string(),
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(!warnings.iter().any(|w| w.field == "theme"));
    }
}
