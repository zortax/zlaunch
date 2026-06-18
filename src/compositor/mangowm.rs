use super::base::{CompositorCapabilities, get_display_title, is_launcher_window};
use super::{Compositor, WindowInfo};
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::io::{BufRead, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

pub struct MangowmCompositor {
    socket_path: PathBuf,
}

impl MangowmCompositor {
    pub fn new() -> Option<Self> {
        Some(Self {
            socket_path: std::env::var("MANGO_INSTANCE_SIGNATURE").ok()?.into(),
        })
    }

    fn send_command(&self, cmd: &str) -> Result<String> {
        let mut stream = UnixStream::connect(&self.socket_path).with_context(|| {
            format!("Failed to connect to mango socket: {:?}", self.socket_path)
        })?;

        stream
            .write_all(cmd.as_bytes())
            .context("Failed to write command to mango socket")?;

        let reader = std::io::BufReader::new(stream);
        let response: String = reader
            .lines()
            .next()
            .ok_or(anyhow!("Failed to read response from mango socket"))?
            .context("Failed to read response from mango socket")?;

        Ok(response)
    }
}

impl Compositor for MangowmCompositor {
    fn name(&self) -> &'static str {
        "MangoWM"
    }

    fn focus_window(&self, window_id: &str) -> Result<()> {
        let cmd = format!("dispatch focusid client,{window_id}\n");
        self.send_command(&cmd)?;
        Ok(())
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        let json_string = self.send_command("get all-clients\n")?;

        let reply: MangowmReply =
            serde_json::from_str(&json_string).context("Failed to parse mango clients JSON")?;

        let mut window_info = Vec::new();
        for window in reply.clients {
            if is_launcher_window(&window.appid) {
                continue;
            }

            let workspace = window.tags.first().copied().unwrap_or(1);

            window_info.push(WindowInfo {
                address: format!("{}", window.id),
                title: get_display_title(&window.title, &window.appid),
                class: window.appid,
                workspace,
                focused: window.is_focused,
                icon_data: None,
            });
        }

        Ok(window_info)
    }

    fn capabilities(&self) -> CompositorCapabilities {
        CompositorCapabilities::full()
    }
}

#[derive(Debug, Deserialize)]
struct MangowmReply {
    clients: Vec<MangowmWindow>,
}

#[derive(Debug, Deserialize)]
struct MangowmWindow {
    id: u64,
    title: String,
    appid: String,
    is_focused: bool,
    tags: Vec<i32>,
}
