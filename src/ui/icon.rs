use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::sync::OnceLock;

// Request higher resolution icons (64px) and let GPUI scale them down to display size.
// This provides natural anti-aliasing as extra pixels are blended during downscaling.
#[cfg(unix)]
const ICON_SIZE: u16 = 64;

lazy_static::lazy_static! {
    static ref ICON_CACHE: Arc<RwLock<HashMap<String, Option<PathBuf>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[cfg(unix)]
static ICON_THEME: OnceLock<Option<String>> = OnceLock::new();

/// Get the configured icon theme from KDE/GTK settings
#[cfg(unix)]
fn get_icon_theme() -> Option<&'static str> {
    ICON_THEME
        .get_or_init(|| {
            // Try KDE settings first
            if let Some(theme) = read_kde_icon_theme() {
                return Some(theme);
            }

            // Try GTK3 settings
            if let Some(theme) = read_gtk3_icon_theme() {
                return Some(theme);
            }

            // Try GTK4 settings
            if let Some(theme) = read_gtk4_icon_theme() {
                return Some(theme);
            }

            None
        })
        .as_deref()
}

#[cfg(unix)]
fn read_kde_icon_theme() -> Option<String> {
    let config_path = dirs::config_dir()?.join("kdeglobals");
    let content = fs::read_to_string(config_path).ok()?;

    let mut in_icons_section = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_icons_section = line == "[Icons]";
            continue;
        }
        if in_icons_section && line.starts_with("Theme=") {
            return Some(line.trim_start_matches("Theme=").to_string());
        }
    }
    None
}

#[cfg(unix)]
fn read_gtk3_icon_theme() -> Option<String> {
    let config_path = dirs::config_dir()?.join("gtk-3.0/settings.ini");
    let content = fs::read_to_string(config_path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("gtk-icon-theme-name=") {
            return Some(line.trim_start_matches("gtk-icon-theme-name=").to_string());
        }
    }
    None
}

#[cfg(unix)]
fn read_gtk4_icon_theme() -> Option<String> {
    let config_path = dirs::config_dir()?.join("gtk-4.0/settings.ini");
    let content = fs::read_to_string(config_path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("gtk-icon-theme-name=") {
            return Some(line.trim_start_matches("gtk-icon-theme-name=").to_string());
        }
    }
    None
}

pub fn resolve_icon_path(icon_name: &str) -> Option<PathBuf> {
    if let Ok(cache) = ICON_CACHE.read()
        && let Some(cached) = cache.get(icon_name)
    {
        return cached.clone();
    }

    let path = resolve_icon_internal(icon_name);

    if let Ok(mut cache) = ICON_CACHE.write() {
        cache.insert(icon_name.to_string(), path.clone());
    }

    path
}

#[cfg(unix)]
fn resolve_icon_internal(icon_name: &str) -> Option<PathBuf> {
    // Absolute path - use directly
    if icon_name.starts_with('/') {
        let path = PathBuf::from(icon_name);
        if path.exists() {
            return Some(path);
        }
        return None;
    }

    // Try configured theme first
    if let Some(theme) = get_icon_theme() {
        let icon = freedesktop_icons::lookup(icon_name)
            .with_size(ICON_SIZE)
            .with_theme(theme)
            .find();

        if icon.is_some() {
            return icon;
        }
    }

    // Fallback to hicolor
    let icon = freedesktop_icons::lookup(icon_name)
        .with_size(ICON_SIZE)
        .with_theme("hicolor")
        .find();

    if icon.is_some() {
        return icon;
    }

    // Last resort: no theme specified
    freedesktop_icons::lookup(icon_name)
        .with_size(ICON_SIZE)
        .find()
}

#[cfg(windows)]
fn resolve_icon_internal(_icon_name: &str) -> Option<PathBuf> {
    // On Windows, icon resolution is not yet implemented.
    // Applications launched from .lnk files will use their embedded icons
    // through the Windows shell.
    None
}
