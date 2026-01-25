//! Shared utilities for mode handlers.
//!
//! This module provides common functionality used across different mode handlers,
//! reducing code duplication for input setup and restoration.
//!
//! # Design Note
//!
//! A formal `ModeHandler` trait was considered but not implemented because:
//! - AiModeHandler is fundamentally different (uses streaming, not ListState)
//! - The existing pattern of similar method signatures works well
//! - LauncherView already handles mode-specific differences cleanly
//!
//! Instead, we provide shared utility functions that mode handlers can use.

use gpui::{Context, Window};
use gpui_component::input::InputState;

/// Default placeholder text for the main launcher view.
pub const DEFAULT_PLACEHOLDER: &str = "Search applications...";

/// Set up input for a list-based mode.
///
/// Clears the input value and sets the placeholder to the mode-specific text.
///
/// # Arguments
/// * `input_state` - The input state to modify
/// * `placeholder` - The placeholder text for this mode (must be static)
/// * `window` - The window context
/// * `cx` - The GPUI context
pub fn setup_list_mode_input(
    input_state: &mut InputState,
    placeholder: &'static str,
    window: &mut Window,
    cx: &mut Context<InputState>,
) {
    input_state.set_value("", window, cx);
    input_state.set_placeholder(placeholder, window, cx);
}

/// Restore input to the default main mode state.
///
/// Clears the input value and restores the default "Search applications..." placeholder.
///
/// # Arguments
/// * `input_state` - The input state to modify
/// * `window` - The window context
/// * `cx` - The GPUI context
pub fn restore_main_input(
    input_state: &mut InputState,
    window: &mut Window,
    cx: &mut Context<InputState>,
) {
    input_state.set_value("", window, cx);
    input_state.set_placeholder(DEFAULT_PLACEHOLDER, window, cx);
}

/// Clear the input value without changing the placeholder.
///
/// Useful when a mode wants to clear input after an action but keep the same placeholder.
///
/// # Arguments
/// * `input_state` - The input state to modify
/// * `window` - The window context
/// * `cx` - The GPUI context
pub fn clear_input_value(
    input_state: &mut InputState,
    window: &mut Window,
    cx: &mut Context<InputState>,
) {
    input_state.set_value("", window, cx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_placeholder() {
        assert_eq!(DEFAULT_PLACEHOLDER, "Search applications...");
    }
}
