pub mod cache;
pub mod entry;
pub mod env;
pub mod exec;
#[cfg(unix)]
pub mod parser;
pub mod scanner;

pub use entry::DesktopEntry;
pub use env::{capture_session_environment, get_session_environment};
pub use exec::launch_application;
pub use scanner::scan_applications;
