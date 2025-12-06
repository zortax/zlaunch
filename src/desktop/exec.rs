use crate::desktop::entry::DesktopEntry;
use crate::desktop::env::get_session_environment;
use std::process::Command;

pub fn launch_application(entry: &DesktopEntry) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        let exec = clean_exec_string(&entry.exec);

        if entry.terminal {
            launch_in_terminal_unix(&exec)?;
        } else {
            launch_detached_unix(&exec)?;
        }
    }

    #[cfg(windows)]
    {
        launch_windows(entry)?;
    }

    Ok(())
}

#[cfg(unix)]
fn clean_exec_string(exec: &str) -> String {
    let mut result = exec.to_string();

    for placeholder in [
        "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k",
    ] {
        result = result.replace(placeholder, "");
    }

    result.trim().to_string()
}

#[cfg(unix)]
fn launch_detached_unix(exec: &str) -> anyhow::Result<()> {
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty exec command");
    }

    let program = parts[0];
    let args = &parts[1..];

    Command::new(program)
        .args(args)
        .env_clear()
        .envs(get_session_environment().iter())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    Ok(())
}

#[cfg(unix)]
fn launch_in_terminal_unix(exec: &str) -> anyhow::Result<()> {
    let terminal = get_terminal_unix()?;

    Command::new(&terminal)
        .arg("-e")
        .arg(exec)
        .env_clear()
        .envs(get_session_environment().iter())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    Ok(())
}

#[cfg(unix)]
fn get_terminal_unix() -> anyhow::Result<String> {
    if let Ok(terminal) = std::env::var("TERMINAL") {
        return Ok(terminal);
    }

    if Command::new("which")
        .arg("xterm")
        .output()
        .is_ok_and(|o| o.status.success())
    {
        return Ok("xterm".to_string());
    }

    anyhow::bail!("No terminal emulator found. Set $TERMINAL environment variable.")
}

#[cfg(windows)]
fn launch_windows(entry: &DesktopEntry) -> anyhow::Result<()> {
    use std::os::windows::process::CommandExt;
    
    // Windows creation flags
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    const DETACHED_PROCESS: u32 = 0x00000008;
    
    let path = &entry.path;
    
    // For .lnk files, use cmd /c start to launch them
    if path.extension().is_some_and(|ext| ext == "lnk") {
        Command::new("cmd")
            .args(["/c", "start", "", path.to_str().unwrap_or("")])
            .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
    } else {
        // For executables, run directly
        Command::new(&entry.exec)
            .creation_flags(DETACHED_PROCESS)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
    }

    Ok(())
}
