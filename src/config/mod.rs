use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::items::ThemeSource;
use crate::ui::theme::LauncherTheme;

/// Embedded bundled themes
#[derive(RustEmbed)]
#[folder = "assets/themes"]
#[include = "*.toml"]
struct BundledThemes;

/// Global config instance (mutable via RwLock)
static CONFIG: RwLock<AppConfig> = RwLock::new(AppConfig::default_const());

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Name of the theme to use
    pub theme: String,
    /// Window width in pixels
    pub window_width: f32,
    /// Window height in pixels
    pub window_height: f32,
    /// Automatically apply blur layer rules on Hyprland
    pub hyprland_auto_blur: bool,
    /// Modules that are disabled
    pub disabled_modules: Option<HashSet<ConfigModule>>,
    /// Enable transparency of the window
    pub enable_transparency: bool,
}

/// Modules enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigModule {
    Ai,
    Emojis,
    Calculator,
    Clipboard,
    Search,
    Themes,
}

impl AppConfig {
    /// Const default for static initialization
    const fn default_const() -> Self {
        Self {
            theme: String::new(),
            window_width: 600.0,
            window_height: 400.0,
            hyprland_auto_blur: true,
            disabled_modules: None,
            enable_transparency: true,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            window_width: 600.0,
            window_height: 400.0,
            hyprland_auto_blur: true,
            disabled_modules: None,
            enable_transparency: true,
        }
    }
}

/// Get the config directory path
fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("zlaunch"))
}

/// Check if the config file exists.
pub fn config_file_exists() -> bool {
    config_dir()
        .map(|p| p.join("config.toml").exists())
        .unwrap_or(false)
}

/// Load application config from ~/.config/zlaunch/config.toml
/// Returns None if the config file doesn't exist
/// Logs warning and returns None if parsing fails
pub fn load_app_config() -> Option<AppConfig> {
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

/// Load a theme by name
/// First checks bundled themes, then user themes in ~/.config/zlaunch/themes/{name}.toml
/// Returns None if the theme is not found
/// Logs warning and returns None if parsing fails
pub fn load_theme(name: &str) -> Option<LauncherTheme> {
    // Special case: "default" theme is defined in code, not a file
    if name == "default" {
        return Some(LauncherTheme::default());
    }

    // First, try to load from bundled themes
    let bundled_filename = format!("{}.toml", name);
    if let Some(bundled_file) = BundledThemes::get(&bundled_filename) {
        match std::str::from_utf8(&bundled_file.data) {
            Ok(content) => match toml::from_str::<LauncherTheme>(content) {
                Ok(mut theme) => {
                    // Ensure the theme name matches
                    theme.name = name.to_string();
                    tracing::info!("Loaded bundled theme '{}'", name);
                    return Some(theme);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse bundled theme '{}': {}", name, e);
                    // Fall through to try user themes
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read bundled theme '{}': {}", name, e);
                // Fall through to try user themes
            }
        }
    }

    // If not found in bundled themes, try user config directory
    let themes_dir = config_dir()?.join("themes");
    let theme_path = themes_dir.join(format!("{}.toml", name));

    if !theme_path.exists() {
        tracing::debug!(
            "Theme '{}' not found in bundled themes or at {:?}",
            name,
            theme_path
        );
        return None;
    }

    match std::fs::read_to_string(&theme_path) {
        Ok(content) => match toml::from_str::<LauncherTheme>(&content) {
            Ok(mut theme) => {
                // Ensure the theme name matches the file name
                theme.name = name.to_string();
                tracing::info!("Loaded user theme '{}' from {:?}", name, theme_path);
                Some(theme)
            }
            Err(e) => {
                tracing::warn!("Failed to parse theme file at {:?}: {}", theme_path, e);
                None
            }
        },
        Err(e) => {
            tracing::warn!("Failed to read theme file at {:?}: {}", theme_path, e);
            None
        }
    }
}

/// List all available themes (both bundled and user themes)
pub fn list_themes() -> Vec<String> {
    let mut themes = Vec::new();

    // Add bundled themes
    for filename in BundledThemes::iter() {
        if let Some(name) = filename.strip_suffix(".toml") {
            themes.push(name.to_string());
        }
    }

    // Add user themes from config directory
    if let Some(themes_dir) = config_dir().map(|p| p.join("themes"))
        && themes_dir.exists()
        && let Ok(entries) = std::fs::read_dir(themes_dir)
    {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str()
                && let Some(name) = filename.strip_suffix(".toml")
                && !themes.contains(&name.to_string())
            {
                themes.push(name.to_string());
            }
        }
    }

    themes.sort();
    themes
}

/// Load the configured theme, falling back to default if anything fails
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

/// Initialize config from file (call once at daemon startup)
pub fn init_config() {
    let loaded = load_app_config().unwrap_or_default();
    let mut config = CONFIG.write().unwrap();
    *config = loaded;
}

/// Get a clone of the current config
pub fn config() -> AppConfig {
    CONFIG.read().unwrap().clone()
}

/// Update config in memory and persist to disk if config file exists
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

/// Save config to file
fn save_config_to_file(config: &AppConfig) -> anyhow::Result<()> {
    let config_path = config_dir()
        .ok_or_else(|| anyhow::anyhow!("No config dir"))?
        .join("config.toml");
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&config_path, content)?;
    tracing::debug!("Saved config to {:?}", config_path);
    Ok(())
}

/// Get the configured window width
pub fn window_width() -> f32 {
    config().window_width
}

/// Get the configured window height
pub fn window_height() -> f32 {
    config().window_height
}

/// List all available themes with their source (bundled or user-defined)
pub fn list_all_themes_with_source() -> Vec<(String, ThemeSource)> {
    let mut themes = Vec::new();

    // Add the implicit default theme (defined in code, not a file)
    themes.push(("default".to_string(), ThemeSource::Bundled));

    // Add bundled themes
    for filename in BundledThemes::iter() {
        if let Some(name) = filename.strip_suffix(".toml") {
            themes.push((name.to_string(), ThemeSource::Bundled));
        }
    }

    // Add user themes from config directory
    if let Some(themes_dir) = config_dir().map(|p| p.join("themes"))
        && themes_dir.exists()
        && let Ok(entries) = std::fs::read_dir(themes_dir)
    {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str()
                && let Some(name) = filename.strip_suffix(".toml")
            {
                // Check if this theme name already exists in bundled themes
                let name_string = name.to_string();
                if !themes.iter().any(|(n, _)| n == &name_string) {
                    themes.push((name_string, ThemeSource::UserDefined(filename.to_string())));
                }
            }
        }
    }

    themes.sort_by(|a, b| a.0.cmp(&b.0));
    themes
}
