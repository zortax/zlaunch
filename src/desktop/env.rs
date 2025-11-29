use std::collections::HashMap;
use std::sync::OnceLock;

#[cfg(unix)]
use std::process::Command;

static SESSION_ENV: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Capture the user session environment at startup.
/// On Unix/Linux, this reads from systemd user session to get the full desktop environment,
/// including theming variables like QT_QPA_PLATFORMTHEME, XDG_CURRENT_DESKTOP, etc.
/// On Windows, this simply captures the current process environment.
pub fn capture_session_environment() {
    SESSION_ENV.get_or_init(|| {
        let mut env = HashMap::new();

        // Start with current process environment
        for (key, value) in std::env::vars() {
            env.insert(key, value);
        }

        // Try to get additional variables from systemd user session (Unix only)
        #[cfg(unix)]
        if let Some(systemd_env) = read_systemd_user_environment() {
            for (key, value) in systemd_env {
                // Only add if not already set (prefer current process env)
                env.entry(key).or_insert(value);
            }
        }

        env
    });
}

/// Get the captured session environment for passing to child processes.
pub fn get_session_environment() -> &'static HashMap<String, String> {
    SESSION_ENV.get_or_init(|| {
        // Fallback if not explicitly initialized
        std::env::vars().collect()
    })
}

/// Read environment variables from systemd user session.
#[cfg(unix)]
fn read_systemd_user_environment() -> Option<HashMap<String, String>> {
    let output = Command::new("systemctl")
        .args(["--user", "show-environment"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut env = HashMap::new();

    for line in stdout.lines() {
        if let Some((key, value)) = line.split_once('=') {
            // systemd may quote values, remove quotes if present
            let value = value.trim_matches('"').trim_matches('\'');
            env.insert(key.to_string(), value.to_string());
        }
    }

    Some(env)
}
