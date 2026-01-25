//! Persistent cache for desktop entries.
//!
//! Provides caching of parsed desktop entries to speed up daemon startup.
//! The cache is stored in XDG cache directory and invalidated when source
//! directories are modified.

mod validation;

use crate::desktop::entry::DesktopEntry;
use crate::desktop::scanner::scan_applications;
use crate::ui::icon::resolve_icon_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::{debug, info, warn};

pub use validation::get_directory_mtimes;

/// Current cache format version.
const CACHE_VERSION: u32 = 1;

/// Cached representation of a desktop entry.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub icon_path: Option<PathBuf>,
    pub comment: Option<String>,
    pub categories: Vec<String>,
    pub terminal: bool,
    pub source_path: PathBuf,
    #[serde(with = "system_time_serde")]
    pub mtime: SystemTime,
}

impl From<CachedEntry> for DesktopEntry {
    fn from(cached: CachedEntry) -> Self {
        DesktopEntry::new(
            cached.id,
            cached.name,
            cached.exec,
            cached.icon,
            cached.icon_path,
            cached.comment,
            cached.categories,
            cached.terminal,
            cached.source_path,
        )
    }
}

impl From<&DesktopEntry> for CachedEntry {
    fn from(entry: &DesktopEntry) -> Self {
        let mtime = fs::metadata(&entry.path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        CachedEntry {
            id: entry.id.clone(),
            name: entry.name.clone(),
            exec: entry.exec.clone(),
            icon: entry.icon.clone(),
            icon_path: entry.icon_path.clone(),
            comment: entry.comment.clone(),
            categories: entry.categories.clone(),
            terminal: entry.terminal,
            source_path: entry.path.clone(),
            mtime,
        }
    }
}

/// The full cache structure stored on disk.
#[derive(Serialize, Deserialize)]
pub struct DesktopEntryCache {
    /// Cache format version for compatibility checks.
    pub version: u32,
    /// Cached desktop entries.
    pub entries: Vec<CachedEntry>,
    /// Modification times of scanned directories.
    #[serde(with = "hashmap_system_time_serde")]
    pub dir_mtimes: HashMap<PathBuf, SystemTime>,
}

impl DesktopEntryCache {
    /// Load cache from disk.
    pub fn load() -> Option<Self> {
        let path = Self::cache_path()?;
        let data = fs::read_to_string(&path).ok()?;
        let cache: Self = serde_json::from_str(&data).ok()?;

        if cache.version != CACHE_VERSION {
            debug!("Cache version mismatch, ignoring");
            return None;
        }

        Some(cache)
    }

    /// Save cache to disk.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::cache_path().ok_or_else(|| anyhow::anyhow!("No cache directory"))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;
        debug!("Saved {} entries to cache", self.entries.len());

        Ok(())
    }

    /// Check if the cache is still valid (no directories have been modified).
    pub fn is_valid(&self) -> bool {
        let current_mtimes = validation::get_directory_mtimes();

        // Check if all directories match
        for (path, cached_mtime) in &self.dir_mtimes {
            match current_mtimes.get(path) {
                Some(current_mtime) => {
                    if current_mtime != cached_mtime {
                        debug!(?path, "Directory modified, cache invalid");
                        return false;
                    }
                }
                None => {
                    debug!(?path, "Directory no longer exists, cache invalid");
                    return false;
                }
            }
        }

        // Check for new directories
        for path in current_mtimes.keys() {
            if !self.dir_mtimes.contains_key(path) {
                debug!(?path, "New directory found, cache invalid");
                return false;
            }
        }

        true
    }

    /// Get the cache file path.
    fn cache_path() -> Option<PathBuf> {
        dirs::cache_dir().map(|d| d.join("zlaunch").join("apps.json"))
    }
}

/// Resolve icon paths for all entries.
fn resolve_all_icon_paths(entries: &mut [DesktopEntry]) {
    for entry in entries.iter_mut() {
        if entry.icon_path.is_none() {
            entry.icon_path = entry.icon.as_ref().and_then(|name| resolve_icon_path(name));
        }
    }
}

/// Load applications with caching.
///
/// Attempts to load from cache first. If the cache is invalid or missing,
/// performs a full scan and saves the result to cache.
pub fn load_applications() -> Vec<DesktopEntry> {
    // Try to load from cache
    if let Some(cache) = DesktopEntryCache::load() {
        if cache.is_valid() {
            info!("Loaded {} applications from cache", cache.entries.len());
            return cache.entries.into_iter().map(DesktopEntry::from).collect();
        }
        debug!("Cache is stale, rescanning");
    }

    // Full scan required
    info!("Scanning for desktop applications...");
    let mut entries = scan_applications();
    resolve_all_icon_paths(&mut entries);
    info!("Found {} applications", entries.len());

    // Save to cache
    let cached_entries: Vec<CachedEntry> = entries.iter().map(CachedEntry::from).collect();
    let dir_mtimes = validation::get_directory_mtimes();

    let cache = DesktopEntryCache {
        version: CACHE_VERSION,
        entries: cached_entries,
        dir_mtimes,
    };

    if let Err(e) = cache.save() {
        warn!("Failed to save application cache: {}", e);
    }

    entries
}

/// Serde support for SystemTime.
mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
        (duration.as_secs(), duration.subsec_nanos()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (secs, nanos): (u64, u32) = Deserialize::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::new(secs, nanos))
    }
}

/// Serde support for HashMap<PathBuf, SystemTime>.
mod hashmap_system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(
        map: &HashMap<PathBuf, SystemTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let converted: HashMap<String, (u64, u32)> = map
            .iter()
            .map(|(k, v)| {
                let duration = v.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
                (
                    k.to_string_lossy().to_string(),
                    (duration.as_secs(), duration.subsec_nanos()),
                )
            })
            .collect();
        converted.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<PathBuf, SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let converted: HashMap<String, (u64, u32)> = Deserialize::deserialize(deserializer)?;
        Ok(converted
            .into_iter()
            .map(|(k, (secs, nanos))| (PathBuf::from(k), UNIX_EPOCH + Duration::new(secs, nanos)))
            .collect())
    }
}
