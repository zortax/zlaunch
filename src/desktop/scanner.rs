use crate::desktop::entry::DesktopEntry;
use crate::desktop::parser::parse_desktop_file;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn scan_applications() -> Vec<DesktopEntry> {
    let dirs = get_xdg_application_dirs();
    let mut entries: HashMap<String, DesktopEntry> = HashMap::new();

    for dir in dirs {
        scan_directory(&dir, &mut entries);
    }

    let mut result: Vec<DesktopEntry> = entries.into_values().collect();
    result.sort_by_key(|a| a.name.to_lowercase());
    result
}

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

fn scan_directory(dir: &PathBuf, entries: &mut HashMap<String, DesktopEntry>) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_directory(&path, entries);
            continue;
        }

        if path.extension().is_some_and(|ext| ext == "desktop")
            && let Some(desktop_entry) = parse_desktop_file(&path)
            && !entries.contains_key(&desktop_entry.id)
        {
            entries.insert(desktop_entry.id.clone(), desktop_entry);
        }
    }
}
