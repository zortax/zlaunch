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
/// - Launcher dimensions outside recommended ranges
/// - Search provider URLs missing the `{query}` placeholder
/// - Invalid trigger formats for search providers
pub fn validate_config(config: &AppConfig) -> Vec<ValidationWarning> {
    let mut warnings = vec![];

    // Validate launcher_size dimensions if set
    let (launcher_w, launcher_h) = config.get_launcher_size();

    if launcher_w < 300.0 {
        warnings.push(ValidationWarning {
            field: "launcher_size".to_string(),
            message: format!(
                "Width {} is below minimum (300). Consider increasing for usability.",
                launcher_w
            ),
        });
    } else if launcher_w > 2000.0 {
        warnings.push(ValidationWarning {
            field: "launcher_size".to_string(),
            message: format!(
                "Width {} exceeds maximum (2000). This may cause display issues.",
                launcher_w
            ),
        });
    }

    if launcher_h < 200.0 {
        warnings.push(ValidationWarning {
            field: "launcher_size".to_string(),
            message: format!(
                "Height {} is below minimum (200). Consider increasing for usability.",
                launcher_h
            ),
        });
    } else if launcher_h > 1500.0 {
        warnings.push(ValidationWarning {
            field: "launcher_size".to_string(),
            message: format!(
                "Height {} exceeds maximum (1500). This may cause display issues.",
                launcher_h
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

    // Validate window_size if set (only relevant when enable_backdrop is true)
    if config.enable_backdrop
        && let Some((w, h)) = config.window_size
    {
        if w < launcher_w {
            warnings.push(ValidationWarning {
                field: "window_size".to_string(),
                message: format!(
                    "window_size width ({}) is smaller than launcher_size width ({}). Will use {}.",
                    w, launcher_w, launcher_w
                ),
            });
        }
        if h < launcher_h {
            warnings.push(ValidationWarning {
                    field: "window_size".to_string(),
                    message: format!(
                        "window_size height ({}) is smaller than launcher_size height ({}). Will use {}.",
                        h, launcher_h, launcher_h
                    ),
                });
        }
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
    fn test_validate_launcher_size_width_too_small() {
        let config = AppConfig {
            launcher_size: Some((100.0, 400.0)),
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(
            warnings
                .iter()
                .any(|w| w.field == "launcher_size" && w.message.contains("Width"))
        );
    }

    #[test]
    fn test_validate_launcher_size_width_too_large() {
        let config = AppConfig {
            launcher_size: Some((3000.0, 400.0)),
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(
            warnings
                .iter()
                .any(|w| w.field == "launcher_size" && w.message.contains("Width"))
        );
    }

    #[test]
    fn test_validate_launcher_size_height_too_small() {
        let config = AppConfig {
            launcher_size: Some((600.0, 50.0)),
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(
            warnings
                .iter()
                .any(|w| w.field == "launcher_size" && w.message.contains("Height"))
        );
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

    #[test]
    fn test_validate_window_size_none() {
        let config = AppConfig {
            window_size: None,
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(!warnings.iter().any(|w| w.field == "window_size"));
    }

    #[test]
    fn test_validate_window_size_larger_than_launcher() {
        let config = AppConfig {
            launcher_size: Some((600.0, 400.0)),
            window_size: Some((1920.0, 1080.0)),
            enable_backdrop: true,
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(!warnings.iter().any(|w| w.field == "window_size"));
    }

    #[test]
    fn test_validate_window_size_width_too_small() {
        let config = AppConfig {
            launcher_size: Some((600.0, 400.0)),
            window_size: Some((400.0, 1080.0)),
            enable_backdrop: true,
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(
            warnings
                .iter()
                .any(|w| w.field == "window_size" && w.message.contains("width"))
        );
    }

    #[test]
    fn test_validate_window_size_height_too_small() {
        let config = AppConfig {
            launcher_size: Some((600.0, 400.0)),
            window_size: Some((1920.0, 300.0)),
            enable_backdrop: true,
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        assert!(
            warnings
                .iter()
                .any(|w| w.field == "window_size" && w.message.contains("height"))
        );
    }

    #[test]
    fn test_validate_window_size_both_too_small() {
        let config = AppConfig {
            launcher_size: Some((600.0, 400.0)),
            window_size: Some((400.0, 300.0)),
            enable_backdrop: true,
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        // Should have two warnings: one for width, one for height
        let window_size_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| w.field == "window_size")
            .collect();
        assert_eq!(window_size_warnings.len(), 2);
    }

    #[test]
    fn test_validate_window_size_ignored_when_backdrop_disabled() {
        // When enable_backdrop is false, window_size should not be validated
        let config = AppConfig {
            launcher_size: Some((600.0, 400.0)),
            window_size: Some((100.0, 100.0)), // Much smaller than launcher
            enable_backdrop: false,
            ..AppConfig::default()
        };
        let warnings = validate_config(&config);
        // Should have no window_size warnings since backdrop is disabled
        assert!(!warnings.iter().any(|w| w.field == "window_size"));
    }
}
