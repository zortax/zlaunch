pub mod cache;
pub mod entry;
pub mod env;
pub mod exec;
pub mod icon_cache;
pub mod parser;
pub mod scanner;
pub mod watcher;

pub use cache::load_applications;
pub use entry::DesktopEntry;
pub use env::{capture_session_environment, get_session_environment};
pub use exec::launch_application;
pub use scanner::scan_applications;
