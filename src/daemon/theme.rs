//! Theme configuration and handling for the daemon.

use crate::error::IpcError;
use gpui::hsla;
use gpui_component::theme::Theme;

/// Handle the SetTheme IPC command.
///
/// Validates the theme exists, updates the config, and syncs the theme cache.
pub fn handle_set_theme(name: &str) -> Result<(), IpcError> {
    // Validate theme exists before updating config
    crate::config::load_theme(name).ok_or_else(|| IpcError::ThemeNotFound(name.to_string()))?;

    // Update config (persists to disk if config file exists)
    crate::config::update_config(|config| {
        config.theme = name.to_string();
    });

    // Sync the theme cache from the updated config
    crate::ui::theme::sync_theme_from_config();

    Ok(())
}

/// Configure the global theme for transparent launcher appearance.
///
/// Sets up transparent backgrounds and minimal borders for the overlay look.
pub fn configure_theme(cx: &mut gpui::App) {
    let theme = Theme::global_mut(cx);
    theme.background = hsla(0.0, 0.0, 0.0, 0.0); // Fully transparent
    theme.window_border = hsla(0.0, 0.0, 0.0, 0.0); // No window border
    theme.border = hsla(0.0, 0.0, 1.0, 0.1); // Subtle separator between search and list
    theme.list_active_border = hsla(0.0, 0.0, 0.0, 0.0); // No selection border
    theme.list_active = hsla(0.0, 0.0, 0.0, 0.0); // Fully transparent - we handle selection ourselves
    theme.list_hover = hsla(0.0, 0.0, 0.0, 0.0); // Fully transparent - we handle hover ourselves
    theme.mono_font_family = "Mononoki Nerd Font Mono".into(); // Monospace font for code blocks
}
