//! Preview components for displaying item details.
//!
//! This module provides reusable preview components for displaying
//! detailed content in preview panels (e.g., clipboard history preview).
//!
//! # Architecture
//!
//! Preview components are separated from list item rendering to allow
//! for more detailed, full-width displays of content. The main components:
//!
//! - `EmptyPreview` - Shown when no item is selected
//! - Color preview - Shows color swatch and color codes (HEX, RGB, HSL)
//! - Image preview - Renders image content
//! - Text preview - Shows text with optional syntax highlighting
//!
//! Currently, most preview logic is in `views/clipboard_rendering.rs`.
//! This module provides utilities that can be used across different
//! preview contexts.

use crate::ui::theme::theme;
use gpui::{Div, SharedString, div, prelude::*};

/// Render an empty state for when no item is selected.
///
/// Shows a centered message indicating no selection.
pub fn render_empty_preview(message: &str) -> Div {
    let t = theme();

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_sm()
                .text_color(t.empty_state_color)
                .child(SharedString::from(message.to_string())),
        )
}

/// Create the base container for a preview panel.
///
/// Sets up standard padding and overflow handling.
pub fn preview_container() -> Div {
    let t = theme();

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .px(t.clipboard.preview_padding)
        .py(t.clipboard.preview_padding)
        .overflow_hidden()
}

/// Create a text preview container with left-aligned content.
pub fn text_preview_container() -> Div {
    let t = theme();

    div()
        .w_full()
        .h_full()
        .flex()
        .items_start()
        .px(t.clipboard.preview_padding)
        .py(t.clipboard.preview_padding)
        .overflow_hidden()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_preview() {
        // Just verify it doesn't panic
        let _ = render_empty_preview("No selection");
    }
}
