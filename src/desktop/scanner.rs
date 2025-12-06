use crate::desktop::entry::DesktopEntry;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(unix)]
use crate::desktop::parser::parse_desktop_file;

pub fn scan_applications() -> Vec<DesktopEntry> {
    let mut entries: HashMap<String, DesktopEntry> = HashMap::new();
    
    #[cfg(unix)]
    {
        let dirs = get_xdg_application_dirs();
        for dir in dirs {
            scan_directory_unix(&dir, &mut entries);
        }
    }

    #[cfg(windows)]
    {
        scan_start_menu(&mut entries);
    }

    let mut result: Vec<DesktopEntry> = entries.into_values().collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    result
}

#[cfg(unix)]
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

#[cfg(unix)]
fn scan_directory_unix(dir: &PathBuf, entries: &mut HashMap<String, DesktopEntry>) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_directory_unix(&path, entries);
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

#[cfg(windows)]
fn scan_start_menu(entries: &mut HashMap<String, DesktopEntry>) {
    // Scan common Start Menu locations
    let start_menu_dirs = get_start_menu_dirs();
    
    for dir in start_menu_dirs {
        scan_directory_windows(&dir, entries);
    }
}

#[cfg(windows)]
fn get_start_menu_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User Start Menu
    if let Some(data_dir) = dirs::data_dir() {
        dirs.push(data_dir.join("Microsoft").join("Windows").join("Start Menu").join("Programs"));
    }

    // Common Start Menu (ProgramData)
    if let Ok(program_data) = std::env::var("ProgramData") {
        dirs.push(PathBuf::from(program_data).join("Microsoft").join("Windows").join("Start Menu").join("Programs"));
    }

    dirs
}

#[cfg(windows)]
fn scan_directory_windows(dir: &PathBuf, entries: &mut HashMap<String, DesktopEntry>) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_directory_windows(&path, entries);
            continue;
        }

        // Parse .lnk (shortcut) files
        if path.extension().is_some_and(|ext| ext == "lnk") {
            if let Some(desktop_entry) = parse_lnk_file(&path)
                && !entries.contains_key(&desktop_entry.id)
            {
                entries.insert(desktop_entry.id.clone(), desktop_entry);
            }
        }
    }
}

#[cfg(windows)]
fn parse_lnk_file(path: &PathBuf) -> Option<DesktopEntry> {
    // Get the name from the filename (without .lnk extension)
    let name = path.file_stem()?.to_str()?.to_string();
    
    // Generate a unique ID based on the path
    let id = path.to_str()?.replace(['\\', '/', ' '], "_");
    
    // Use the .lnk file path as the exec command
    // Windows will handle launching it properly
    let exec = path.to_str()?.to_string();
    
    Some(DesktopEntry::new(
        id,
        name,
        exec,
        None,      // icon - Windows will use the shortcut's icon
        None,      // icon_path - resolved separately
        None,      // comment
        vec![],    // categories
        false,     // terminal
        path.clone(),
    ))
}
