use crate::desktop::entry::DesktopEntry;
use crate::process;

pub fn launch_application(entry: &DesktopEntry) -> anyhow::Result<()> {
    let exec = clean_exec_string(&entry.exec);

    if entry.terminal {
        process::launch_in_terminal(&exec)?;
    } else {
        process::launch_exec(&exec)?;
    }

    Ok(())
}

fn clean_exec_string(exec: &str) -> String {
    let mut result = exec.to_string();

    for placeholder in [
        "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k",
    ] {
        result = result.replace(placeholder, "");
    }

    result.trim().to_string()
}
