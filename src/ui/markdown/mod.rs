//! Markdown rendering support for AI responses.
//!
//! Uses gpui-component's TextView for full GFM markdown support including:
//! - Paragraphs, headings (H1-H6)
//! - Bold, italic, strikethrough
//! - Inline code, code blocks with syntax highlighting
//! - Links, images
//! - Ordered/unordered lists (nested)
//! - Blockquotes, tables, horizontal rules

use std::sync::Arc;

use gpui::{App, IntoElement, SharedString, StyleRefinement, Window, div, prelude::*, px, rems};
use gpui_component::highlighter::HighlightTheme;
use gpui_component::text::{TextView, TextViewStyle};

use crate::ui::theme::theme;

/// Render markdown text using gpui-component's TextView.
///
/// Supports full GFM markdown:
/// - Paragraphs, headings (H1-H6)
/// - Bold, italic, strikethrough
/// - Inline code, code blocks with syntax highlighting
/// - Links, images
/// - Ordered/unordered lists (nested)
/// - Blockquotes, tables, horizontal rules
pub fn render_markdown(text: &str, window: &mut Window, cx: &mut App) -> impl IntoElement {
    render_markdown_with_id("ai-response-markdown", text, window, cx)
}

/// Render markdown text with a custom element ID.
pub fn render_markdown_with_id(
    id: impl Into<SharedString>,
    text: &str,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    let t = theme();

    // Determine if dark theme based on background lightness
    let is_dark = t.window_background.l < 0.5;

    let highlight_theme = if is_dark {
        HighlightTheme::default_dark()
    } else {
        HighlightTheme::default_light()
    };

    let code_block_bg = t.item_background_selected;
    let code_block_radius = t.markdown.code_block_radius;

    // Use smaller font sizes to match the rest of the UI (text_sm is ~12px)
    let style = TextViewStyle {
        paragraph_gap: rems(1.5), // Generous spacing between sections and around separators
        heading_base_font_size: px(14.0),
        heading_font_size: Some(Arc::new(|level, _base| match level {
            1 => px(16.0),
            2 => px(14.0),
            3 => px(13.0),
            _ => px(12.0),
        })),
        highlight_theme,
        code_block: StyleRefinement::default()
            .bg(code_block_bg)
            .rounded(code_block_radius),
        is_dark,
    };

    let id: SharedString = id.into();
    let text: SharedString = text.to_string().into();

    // Wrap in a container with text_sm for consistent small font size
    div()
        .text_sm()
        .child(TextView::markdown(id, text).style(style).selectable(true))
}
