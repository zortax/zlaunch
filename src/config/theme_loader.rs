//! Theme loading utilities.

use crate::items::ThemeSource;
use crate::ui::theme::LauncherTheme;
use rust_embed::RustEmbed;
use std::path::PathBuf;

/// Embedded bundled themes.
#[derive(RustEmbed)]
#[folder = "assets/themes"]
#[include = "*.toml"]
struct BundledThemes;

/// Get the config directory path.
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("zlaunch"))
}

/// Load a theme by name.
///
/// First checks bundled themes, then user themes in `~/.config/zlaunch/themes/{name}.toml`.
/// Returns `None` if the theme is not found.
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

/// List all available themes (both bundled and user themes).
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

/// List all available themes with their source (bundled or user-defined).
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
