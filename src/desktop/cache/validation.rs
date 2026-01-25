//! Cache validation utilities.
//!
//! Provides functions for checking directory modification times
//! to determine cache validity.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Get modification times for all XDG application directories.
///
/// Returns a map of directory paths to their modification times.
/// Directories that don't exist are not included.
pub fn get_directory_mtimes() -> HashMap<PathBuf, SystemTime> {
    let mut mtimes = HashMap::new();

    for dir in get_xdg_application_dirs() {
        if let Ok(metadata) = fs::metadata(&dir) {
            if let Ok(mtime) = metadata.modified() {
                mtimes.insert(dir, mtime);
            }
        }
    }

    mtimes
}

/// Get the list of XDG application directories to scan.
fn get_xdg_application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = dirs::data_local_dir() {
        dirs.push(data_home.join("applications"));
    }

    if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg_dirs.split(':') {
            dirs.push(PathBuf::from(dir).join("applications"));
        }
    } else {
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/usr/share/applications"));
    }

    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_directory_mtimes() {
        let mtimes = get_directory_mtimes();
        // Just verify it runs without panic
        let _ = mtimes;
    }
}
