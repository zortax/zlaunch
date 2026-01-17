//! Persistent icon cache for desktop entries.
//!
//! Caches resolved icon paths to disk to avoid repeated lookups
//! on daemon startup. The cache is invalidated when the icon theme changes.

use crate::ui::icon::{get_current_theme, resolve_icon_path};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Persistent cache for icon paths.
#[derive(Serialize, Deserialize)]
pub struct IconCache {
    /// The icon theme this cache was built with.
    pub theme: String,
    /// Map from icon name to resolved path (None if not found).
    pub entries: HashMap<String, Option<PathBuf>>,
}

impl IconCache {
    /// Load icon cache from disk.
    pub fn load() -> Option<Self> {
        let path = Self::cache_path()?;
        let data = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Save icon cache to disk.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::cache_path().ok_or_else(|| anyhow::anyhow!("No cache directory"))?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;
        debug!("Saved {} icon paths to cache", self.entries.len());

        Ok(())
    }

    /// Check if the cache is valid for the current theme.
    pub fn is_valid(&self, current_theme: &str) -> bool {
        self.theme == current_theme
    }

    /// Resolve an icon path, using cache if available.
    pub fn resolve(&mut self, icon_name: &str) -> Option<PathBuf> {
        if let Some(cached) = self.entries.get(icon_name) {
            return cached.clone();
        }

        let resolved = resolve_icon_path(icon_name);
        self.entries.insert(icon_name.to_string(), resolved.clone());
        resolved
    }

    /// Create a new empty cache for the given theme.
    pub fn new(theme: String) -> Self {
        Self {
            theme,
            entries: HashMap::new(),
        }
    }

    /// Get the cache file path.
    fn cache_path() -> Option<PathBuf> {
        dirs::cache_dir().map(|d| d.join("zlaunch").join("icons.json"))
    }
}

/// Load or create an icon cache for the current theme.
pub fn get_icon_cache() -> IconCache {
    let current_theme = get_current_theme();

    if let Some(cache) = IconCache::load() {
        if cache.is_valid(&current_theme) {
            debug!(
                "Using cached icon paths ({} entries, theme: {})",
                cache.entries.len(),
                current_theme
            );
            return cache;
        }
        debug!("Icon cache theme mismatch, rebuilding");
    }

    IconCache::new(current_theme)
}

/// Save the icon cache, logging any errors.
pub fn save_icon_cache(cache: &IconCache) {
    if let Err(e) = cache.save() {
        warn!("Failed to save icon cache: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache() {
        let cache = IconCache::new("hicolor".to_string());
        assert_eq!(cache.theme, "hicolor");
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_is_valid() {
        let cache = IconCache::new("hicolor".to_string());
        assert!(cache.is_valid("hicolor"));
        assert!(!cache.is_valid("breeze"));
    }
}
