//! Process execution utilities for launching detached applications.
//!
//! This module provides safe abstractions for spawning processes that outlive
//! the launcher daemon. All spawned processes are detached using `setsid()`
//! to create a new session, preventing them from being killed when the daemon exits.

use crate::desktop::env::get_session_environment;
use crate::error::ProcessError;
use std::ffi::OsStr;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

/// Builder for creating detached processes.
///
/// A detached process runs in its own session (via `setsid()`) and survives
/// when the parent (launcher daemon) exits. All stdio is redirected to null.
///
/// # Example
/// ```ignore
/// use zlaunch::process::DetachedProcess;
///
/// DetachedProcess::new("firefox")
///     .arg("https://example.com")
///     .spawn()?;
/// ```
pub struct DetachedProcess {
    command: Command,
    use_session_env: bool,
    shell_command: Option<String>,
}

impl DetachedProcess {
    /// Create a new detached process builder for the given program.
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            command: Command::new(program),
            use_session_env: false,
            shell_command: None,
        }
    }

    /// Create a detached process that runs a shell command.
    ///
    /// The command will be executed via `sh -c "command"`.
    pub fn shell<S: Into<String>>(command: S) -> Self {
        let cmd = command.into();
        Self {
            command: Command::new("sh"),
            use_session_env: false,
            shell_command: Some(cmd),
        }
    }

    /// Add an argument to the process.
    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.command.arg(arg);
        self
    }

    /// Add multiple arguments to the process.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    /// Use the captured session environment for the spawned process.
    ///
    /// This clears the environment and sets it to the session environment
    /// captured at daemon startup, which includes theming variables like
    /// `QT_QPA_PLATFORMTHEME`, `XDG_CURRENT_DESKTOP`, etc.
    pub fn with_session_env(mut self) -> Self {
        self.use_session_env = true;
        self
    }

    /// Spawn the detached process.
    ///
    /// The spawned process:
    /// - Runs in a new session (calls `setsid()`)
    /// - Has stdin/stdout/stderr redirected to /dev/null
    /// - Survives when the parent process exits
    ///
    /// # Safety
    /// This function uses `pre_exec` to call `libc::setsid()`, which is
    /// async-signal-safe and therefore safe to use in this context.
    pub fn spawn(mut self) -> Result<(), ProcessError> {
        // Handle shell commands
        if let Some(cmd) = &self.shell_command {
            self.command.args(["-c", cmd]);
        }

        // Set up environment
        if self.use_session_env {
            self.command.env_clear();
            self.command.envs(get_session_environment().iter());
        }

        // Redirect stdio to null
        self.command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // SAFETY: setsid() is async-signal-safe and creates a new session,
        // detaching the child from the parent's process group so it survives
        // when the daemon exits.
        unsafe {
            self.command.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }

        self.command.spawn().map_err(ProcessError::SpawnFailed)?;

        Ok(())
    }
}

/// Launch an application with the given executable string.
///
/// The exec string is split on whitespace to extract program and arguments.
/// Empty exec strings return an error.
pub fn launch_exec(exec: &str) -> Result<(), ProcessError> {
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if parts.is_empty() {
        return Err(ProcessError::EmptyCommand);
    }

    let program = parts[0];
    let args = &parts[1..];

    DetachedProcess::new(program)
        .args(args.iter().copied())
        .with_session_env()
        .spawn()
}

/// Launch an application in a terminal emulator.
///
/// Uses the `$TERMINAL` environment variable, falling back to `xterm`.
pub fn launch_in_terminal(exec: &str) -> Result<(), ProcessError> {
    let terminal = get_terminal()?;

    DetachedProcess::new(&terminal)
        .arg("-e")
        .arg(exec)
        .with_session_env()
        .spawn()
}

/// Open a URL using the system default handler (`xdg-open`).
pub fn open_url(url: &str) -> Result<(), ProcessError> {
    DetachedProcess::new("xdg-open").arg(url).spawn()
}

/// Execute a shell command in a detached process.
pub fn run_shell_command(command: &str) -> Result<(), ProcessError> {
    DetachedProcess::shell(command).spawn()
}

/// Get the terminal emulator to use.
fn get_terminal() -> Result<String, ProcessError> {
    if let Ok(terminal) = std::env::var("TERMINAL") {
        return Ok(terminal);
    }

    // Check for xterm
    if std::process::Command::new("which")
        .arg("xterm")
        .output()
        .is_ok_and(|o| o.status.success())
    {
        return Ok("xterm".to_string());
    }

    Err(ProcessError::NoTerminal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_exec_empty() {
        let result = launch_exec("");
        assert!(matches!(result, Err(ProcessError::EmptyCommand)));
    }

    #[test]
    fn test_launch_exec_whitespace_only() {
        let result = launch_exec("   ");
        assert!(matches!(result, Err(ProcessError::EmptyCommand)));
    }
}
