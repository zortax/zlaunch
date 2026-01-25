//! IPC module for daemon communication using tarpc.

pub mod client;
pub mod commands;
pub mod server;

pub use commands::{ThemeInfo, ZlaunchServiceClient};
pub use server::{IpcServerHandle, get_socket_path, prepare_socket, start_server};
