use super::base::{get_display_title, is_launcher_window, CompositorCapabilities};
use super::{Compositor, WindowInfo};
use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use std::io::{BufRead, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

pub struct NiriCompositor {
    socket_path: PathBuf,
}

impl NiriCompositor {
    pub fn new() -> Option<Self> {
        Some(Self {
            socket_path: std::env::var("NIRI_SOCKET").ok()?.into(),
        })
    }

    fn send_command(&self, cmd: &str) -> Result<String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .with_context(|| format!("Failed to connect to Niri socket: {:?}", self.socket_path))?;

        stream
            .write_all(cmd.as_bytes())
            .context("Failed to write command to Niri socket")?;

        let reader = std::io::BufReader::new(stream);
        let response: String = reader
            .lines()
            .next()
            .ok_or(anyhow!("Failed to read response from Niri socket"))?
            .context("Failed to read response from Niri socket")?;

        Ok(response)
    }
}

impl Compositor for NiriCompositor {
    fn name(&self) -> &'static str {
        "Niri"
    }

    fn focus_window(&self, window_id: &str) -> Result<()> {
        let newline = "\n";
        let cmd = format!(r#"{{"Action":{{"FocusWindow":{{"id":{window_id}}}}}}}{newline}"#);
        self.send_command(&cmd)?;
        Ok(())
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        let json_string = self.send_command("\"Windows\"\n")?;

        let niri_result: std::result::Result<NiriWindowReply, serde_json::Value> =
            serde_json::from_str(&json_string).context("Failed to parse Niri clients JSON")?;

        let Ok(niri_reply) = niri_result else {
            bail!("Niri returned an error to Windows request");
        };

        let mut window_info = Vec::new();
        for window in niri_reply.windows {
            if is_launcher_window(&window.app_id) {
                continue;
            }

            window_info.push(WindowInfo {
                address: format!("{}", window.id),
                title: get_display_title(&window.title, &window.app_id),
                class: window.app_id,
                workspace: window.workspace_id as i32,
                focused: window.is_focused,
            });
        }

        Ok(window_info)
    }

    fn capabilities(&self) -> CompositorCapabilities {
        CompositorCapabilities::full()
    }
}

#[derive(Debug, Deserialize)]
struct NiriWindowReply {
    #[serde(rename = "Windows")]
    windows: Vec<NiriWindow>,
}

#[derive(Debug, Deserialize)]
struct NiriWindow {
    id: i64,
    title: String,
    app_id: String,
    workspace_id: i64,
    is_focused: bool,
}
