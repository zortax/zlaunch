//! Configuration management for zlaunch.
//!
//! This module provides configuration loading, theme management, and
//! application settings. Configuration is loaded from `~/.config/zlaunch/config.toml`
//! and validated on startup.
//!
//! # Modules
//!
//! - `service` - Configuration loading, caching, and persistence
//! - `theme_loader` - Theme discovery and loading
//! - `types` - Configuration type definitions
//! - `validation` - Configuration validation utilities

mod service;
mod theme_loader;
mod types;
pub mod validation;

// Re-export types
pub use types::{AppConfig, ConfigModule, ConfigSearchProvider, LauncherMode};

// Re-export service functions
pub use service::{
    config, config_file_exists, get_combined_modules, get_default_modes, init_config,
    load_configured_theme, update_config, window_height, window_width, ConfigProvider,
    ConfigService,
};

// Re-export theme functions
pub use theme_loader::{list_all_themes_with_source, list_themes, load_theme};
