//! Desktop file parser for Unix/Linux systems.
//! This module is only compiled on Unix platforms.

use crate::desktop::entry::DesktopEntry;
use freedesktop_desktop_entry::DesktopEntry as FdEntry;
use std::path::Path;

pub fn parse_desktop_file(path: &Path) -> Option<DesktopEntry> {
    let content = std::fs::read_to_string(path).ok()?;
    let fd_entry = FdEntry::from_str(path, &content, None::<&[&str]>).ok()?;

    let locales: &[&str] = &[];
    let name = fd_entry.name(locales)?.to_string();
    let exec = fd_entry.exec()?.to_string();

    if fd_entry.no_display() {
        return None;
    }

    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let icon = fd_entry.icon().map(|s| s.to_string());
    let comment = fd_entry.comment(locales).map(|s| s.to_string());

    let categories: Vec<String> = fd_entry
        .categories()
        .map(|cats| cats.into_iter().map(|c| c.to_string()).collect())
        .unwrap_or_default();

    let terminal = fd_entry.terminal();

    // icon_path is resolved later in cache.rs after all entries are loaded
    Some(DesktopEntry::new(
        id,
        name,
        exec,
        icon,
        None,
        comment,
        categories,
        terminal,
        path.to_path_buf(),
    ))
}
