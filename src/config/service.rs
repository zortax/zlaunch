//! Configuration service for managing application config.

use std::sync::{Once, RwLock};

use crate::ui::theme::LauncherTheme;

use super::theme_loader::{config_dir, load_theme};
use super::types::{AppConfig, ConfigModule, LauncherMode};

/// Global config instance (mutable via RwLock).
static CONFIG: RwLock<AppConfig> = RwLock::new(AppConfig::default_const());

/// One-time warning for deprecated disabled_modules option.
static DISABLED_MODULES_WARNING: Once = Once::new();

/// Trait for providing configuration access.
///
/// This trait allows for dependency injection in tests and enables
/// different components to access configuration without depending on
/// the global state directly.
pub trait ConfigProvider: Send + Sync {
    /// Get a clone of the current configuration.
    fn get(&self) -> AppConfig;

    /// Update the configuration.
    fn update<F: FnOnce(&mut AppConfig)>(&self, f: F);

    /// Get the current theme.
    fn theme(&self) -> LauncherTheme;
}

/// Configuration service that owns and manages application config.
///
/// Currently wraps the global static, but provides an abstraction
/// that can be used for dependency injection in the future.
pub struct ConfigService;

impl ConfigService {
    /// Create a new config service.
    pub fn new() -> Self {
        ConfigService
    }
}

impl ConfigProvider for ConfigService {
    fn get(&self) -> AppConfig {
        config()
    }

    fn update<F: FnOnce(&mut AppConfig)>(&self, f: F) {
        update_config(f);
    }

    fn theme(&self) -> LauncherTheme {
        load_configured_theme()
    }
}

impl Default for ConfigService {
    fn default() -> Self {
        Self::new()
    }
}

// === Global functions (backwards compatible) ===

/// Check if the config file exists.
pub fn config_file_exists() -> bool {
    config_dir()
        .map(|p| p.join("config.toml").exists())
        .unwrap_or(false)
}

/// Load application config from `~/.config/zlaunch/config.toml`.
///
/// Returns `None` if the config file doesn't exist.
/// Logs warning and returns `None` if parsing fails.
fn load_app_config() -> Option<AppConfig> {
    let config_path = config_dir()?.join("config.toml");

    if !config_path.exists() {
        tracing::debug!("Config file not found at {:?}, using defaults", config_path);
        return None;
    }

    match std::fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str::<AppConfig>(&content) {
            Ok(config) => {
                tracing::info!("Loaded app config from {:?}", config_path);
                Some(config)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to parse config file at {:?}: {}, using defaults",
                    config_path,
                    e
                );
                None
            }
        },
        Err(e) => {
            tracing::warn!(
                "Failed to read config file at {:?}: {}, using defaults",
                config_path,
                e
            );
            None
        }
    }
}

/// Initialize config from file (call once at daemon startup).
///
/// This function loads the configuration and validates it, logging
/// any warnings for invalid or unusual values.
pub fn init_config() {
    let loaded = load_app_config().unwrap_or_default();

    // Validate configuration and log warnings
    let warnings = super::validation::validate_config(&loaded);
    for warning in warnings {
        tracing::warn!("Config validation: {} - {}", warning.field, warning.message);
    }

    let mut config = CONFIG.write().unwrap();
    *config = loaded;
}

/// Get a clone of the current config.
pub fn config() -> AppConfig {
    CONFIG.read().unwrap().clone()
}

/// Update config in memory and persist to disk if config file exists.
pub fn update_config(f: impl FnOnce(&mut AppConfig)) {
    let mut config = CONFIG.write().unwrap();
    f(&mut config);

    // Only save if config file already exists
    if config_file_exists()
        && let Err(e) = save_config_to_file(&config)
    {
        tracing::warn!("Failed to save config: {}", e);
    }
}

/// Save config to file.
fn save_config_to_file(config: &AppConfig) -> anyhow::Result<()> {
    let config_path = config_dir()
        .ok_or_else(|| anyhow::anyhow!("No config dir"))?
        .join("config.toml");
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&config_path, content)?;
    tracing::debug!("Saved config to {:?}", config_path);
    Ok(())
}

/// Get the configured window width.
pub fn window_width() -> f32 {
    config().window_width
}

/// Get the configured window height.
pub fn window_height() -> f32 {
    config().window_height
}

/// Load the configured theme, falling back to default if anything fails.
pub fn load_configured_theme() -> LauncherTheme {
    // Get theme name from cached config
    let theme_name = config().theme;

    // If a non-default theme is requested, try to load it
    if theme_name != "default" {
        if let Some(theme) = load_theme(&theme_name) {
            return theme;
        }
        tracing::warn!(
            "Failed to load theme '{}', falling back to default",
            theme_name
        );
    }

    // Use default theme
    LauncherTheme::default()
}

/// Get the modules to include in combined view (ordered).
///
/// Handles backwards compatibility with deprecated `disabled_modules`.
pub fn get_combined_modules() -> Vec<ConfigModule> {
    let cfg = config();

    // Priority: disabled_modules (deprecated) > combined_modules > all modules
    if let Some(disabled) = &cfg.disabled_modules {
        // Warn about deprecated option (only once)
        DISABLED_MODULES_WARNING.call_once(|| {
            if cfg.combined_modules.is_some() {
                tracing::warn!(
                    "Both 'disabled_modules' and 'combined_modules' are set. \
                     Using deprecated 'disabled_modules' (combined_modules will be ignored)"
                );
            } else {
                tracing::warn!("'disabled_modules' is deprecated. Use 'combined_modules' instead.");
            }
        });
        // Old behavior: all modules except disabled ones
        ConfigModule::all()
            .into_iter()
            .filter(|m| !disabled.contains(m))
            .collect()
    } else if let Some(combined) = &cfg.combined_modules {
        // New behavior: explicit ordered list
        combined.clone()
    } else {
        // Default: all modules
        ConfigModule::all()
    }
}

/// Get the default modes to cycle through.
///
/// Returns configured modes or `[Combined]` as default.
pub fn get_default_modes() -> Vec<LauncherMode> {
    config()
        .default_modes
        .as_ref()
        .map(|names| {
            names
                .iter()
                .filter_map(|s| LauncherMode::parse_str(s))
                .collect()
        })
        .filter(|modes: &Vec<LauncherMode>| !modes.is_empty())
        .unwrap_or_else(|| vec![LauncherMode::Combined])
}
