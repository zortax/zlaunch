//! tarpc client for communicating with the daemon.

use crate::config::LauncherMode;
use crate::ipc::commands::{ThemeInfo, ZlaunchServiceClient};
use crate::ipc::server::get_socket_path;
use tarpc::client;
use tarpc::context;
use tarpc::tokio_serde::formats::Json;
use tokio::net::UnixStream;
use tokio_util::codec::LengthDelimitedCodec;

/// Check if the daemon is running.
pub fn is_daemon_running() -> bool {
    let socket_path = get_socket_path();
    std::os::unix::net::UnixStream::connect(&socket_path).is_ok()
}

/// Create a tarpc client connected to the daemon.
async fn connect() -> anyhow::Result<ZlaunchServiceClient> {
    let socket_path = get_socket_path();
    let stream = UnixStream::connect(&socket_path).await?;

    let framed = tokio_util::codec::Framed::new(stream, LengthDelimitedCodec::new());
    let transport = tarpc::serde_transport::new(framed, Json::default());

    let client = ZlaunchServiceClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}

/// Show the launcher window with optional modes.
pub fn show(modes: Option<Vec<LauncherMode>>) -> anyhow::Result<()> {
    run_async(async {
        let client = connect().await?;
        Ok(client.show(context::current(), modes).await??)
    })
}

/// Hide the launcher window.
pub fn hide() -> anyhow::Result<()> {
    run_async(async {
        let client = connect().await?;
        Ok(client.hide(context::current()).await??)
    })
}

/// Toggle the launcher window visibility with optional modes.
pub fn toggle(modes: Option<Vec<LauncherMode>>) -> anyhow::Result<()> {
    run_async(async {
        let client = connect().await?;
        Ok(client.toggle(context::current(), modes).await??)
    })
}

/// Quit the daemon.
pub fn quit() -> anyhow::Result<()> {
    run_async(async {
        let client = connect().await?;
        Ok(client.quit(context::current()).await??)
    })
}

/// Reload the daemon (fully restart the process).
pub fn reload() -> anyhow::Result<()> {
    run_async(async {
        let client = connect().await?;
        Ok(client.reload(context::current()).await??)
    })
}

/// List all available themes.
pub fn list_themes() -> anyhow::Result<Vec<ThemeInfo>> {
    run_async(async {
        let client = connect().await?;
        Ok(client.list_themes(context::current()).await?)
    })
}

/// Get the current theme name.
pub fn get_current_theme() -> anyhow::Result<String> {
    run_async(async {
        let client = connect().await?;
        Ok(client.get_current_theme(context::current()).await?)
    })
}

/// Set the active theme.
pub fn set_theme(name: &str) -> anyhow::Result<()> {
    let name = name.to_string();
    run_async(async {
        let client = connect().await?;
        Ok(client.set_theme(context::current(), name).await??)
    })
}

/// Run an async operation synchronously using a temporary tokio runtime.
fn run_async<F, T>(future: F) -> anyhow::Result<T>
where
    F: std::future::Future<Output = anyhow::Result<T>>,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(future)
}
