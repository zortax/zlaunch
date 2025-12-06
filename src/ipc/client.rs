use crate::ipc::commands::{Command, Response};
use std::io::{Read, Write};

#[cfg(unix)]
use crate::ipc::server::get_socket_path;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
use crate::ipc::server::get_socket_addr;
#[cfg(windows)]
use std::net::TcpStream;

#[cfg(unix)]
pub fn send_command(cmd: Command) -> anyhow::Result<Response> {
    let socket_path = get_socket_path();
    let mut stream = UnixStream::connect(&socket_path)?;

    let msg = serde_json::to_vec(&cmd)?;
    stream.write_all(&msg)?;

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf)?;
    let response: Response = serde_json::from_slice(&buf[..n])?;

    Ok(response)
}

#[cfg(windows)]
pub fn send_command(cmd: Command) -> anyhow::Result<Response> {
    let mut stream = TcpStream::connect(get_socket_addr())?;

    let msg = serde_json::to_vec(&cmd)?;
    stream.write_all(&msg)?;

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf)?;
    let response: Response = serde_json::from_slice(&buf[..n])?;

    Ok(response)
}

#[cfg(unix)]
pub fn is_daemon_running() -> bool {
    let socket_path = get_socket_path();
    UnixStream::connect(&socket_path).is_ok()
}

#[cfg(windows)]
pub fn is_daemon_running() -> bool {
    TcpStream::connect(get_socket_addr()).is_ok()
}
