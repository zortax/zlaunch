//! Rendering functions for clipboard history view.

use crate::assets::PhosphorIcon;
use crate::clipboard::{ClipboardContent, ClipboardItem};
use crate::ui::theme::theme;
use crate::ui::utils::color::{parse_color, Color};
use gpui::{Div, ElementId, SharedString, Stateful, div, img, prelude::*, px, svg};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Render a clipboard item in the list.
pub fn render_clipboard_item(item: &ClipboardItem, selected: bool, row: usize) -> Stateful<Div> {
    let t = theme();

    let bg = if selected {
        t.item_background_selected
    } else {
        t.item_background
    };

    // Format timestamp
    let timestamp_str = format_timestamp(&item.timestamp);

    // Get preview text
    let preview = get_item_preview(item);

    div()
        .id(ElementId::NamedInteger("clipboard-item".into(), row as u64))
        .ml(px(0.0))
        .mr(t.item_margin_x)
        .my(t.item_margin_y)
        .px(t.item_padding_x)
        .py(t.item_padding_y)
        .bg(bg)
        .rounded(t.item_border_radius)
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        // Icon (type-specific)
        .child(render_item_icon(item))
        // Content: preview text and timestamp
        .child(
            div()
                .flex_1()
                .h(t.item_content_height)
                .flex()
                .flex_col()
                .justify_center()
                .overflow_hidden()
                .child(
                    div()
                        .w_full()
                        .text_sm()
                        .line_height(t.item_title_line_height)
                        .text_color(t.item_title_color)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(SharedString::from(preview)),
                )
                .child(
                    div()
                        .w_full()
                        .text_xs()
                        .h(t.layout.item_description_height)
                        .text_color(t.item_description_color)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(SharedString::from(timestamp_str)),
                ),
        )
}

/// Get preview text for a clipboard item.
fn get_item_preview(item: &ClipboardItem) -> String {
    item.preview()
}

/// Render the appropriate icon for a clipboard item.
fn render_item_icon(item: &ClipboardItem) -> Div {
    let t = theme();

    // Check if this is text content
    if let ClipboardContent::Text(text) = &item.content {
        // Check if it's a color
        if let Some(color) = parse_color(text) {
            let (h, s, l) = color.to_hsl();
            // Render a small colored circle with background box
            return div()
                .w(t.icon_size)
                .h(t.icon_size)
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .bg(t.icon_placeholder_background)
                .rounded_sm()
                .child(
                    div()
                        .w(t.clipboard.color_icon_size)
                        .h(t.clipboard.color_icon_size)
                        .rounded(t.clipboard.color_icon_size * 0.5)
                        .bg(gpui::hsla(
                            h as f32 / 360.0,
                            s as f32 / 100.0,
                            l as f32 / 100.0,
                            color.a as f32 / 255.0,
                        ))
                        .border_1()
                        .border_color(t.window_border),
                );
        }

        // Check if it's a file:// URL
        if let Some(path) = parse_file_url(text) {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if matches!(
                    ext_lower.as_str(),
                    "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg"
                ) {
                    return render_icon_container(PhosphorIcon::FileImage);
                } else if matches!(
                    ext_lower.as_str(),
                    "txt"
                        | "md"
                        | "rs"
                        | "py"
                        | "js"
                        | "ts"
                        | "json"
                        | "yaml"
                        | "yml"
                        | "toml"
                        | "html"
                        | "css"
                        | "sh"
                ) {
                    return render_icon_container(PhosphorIcon::FileText);
                } else {
                    return render_icon_container(PhosphorIcon::File);
                }
            }
            return render_icon_container(PhosphorIcon::File);
        }

        // Default to clipboard text icon
        return render_icon_container(PhosphorIcon::ClipboardText);
    }

    // Determine icon based on content type
    let icon = match &item.content {
        ClipboardContent::Text(_) => PhosphorIcon::ClipboardText, // Already handled above
        ClipboardContent::Image { .. } => PhosphorIcon::Image,
        ClipboardContent::FilePaths(paths) => {
            if paths.len() == 1 {
                // Check file extension for specific icons
                if let Some(ext) = paths[0].extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(
                        ext_lower.as_str(),
                        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg"
                    ) {
                        PhosphorIcon::FileImage
                    } else if matches!(
                        ext_lower.as_str(),
                        "txt"
                            | "md"
                            | "rs"
                            | "py"
                            | "js"
                            | "ts"
                            | "json"
                            | "yaml"
                            | "yml"
                            | "toml"
                            | "html"
                            | "css"
                            | "sh"
                    ) {
                        PhosphorIcon::FileText
                    } else {
                        PhosphorIcon::File
                    }
                } else {
                    PhosphorIcon::File
                }
            } else {
                PhosphorIcon::File
            }
        }
        ClipboardContent::RichText { .. } => PhosphorIcon::ClipboardText,
    };

    render_icon_container(icon)
}

/// Render icon container matching main item style.
fn render_icon_container(icon: PhosphorIcon) -> Div {
    let t = theme();
    div()
        .w(t.icon_size)
        .h(t.icon_size)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(t.icon_placeholder_background)
        .rounded_sm()
        .child(
            svg()
                .path(icon.path())
                .size_4()
                .text_color(t.icon_placeholder_color),
        )
}

/// Format a SystemTime as a relative or absolute timestamp.
fn format_timestamp(time: &SystemTime) -> String {
    let now = SystemTime::now();
    if let Ok(duration) = now.duration_since(*time) {
        let secs = duration.as_secs();
        if secs < 60 {
            return "Just now".to_string();
        } else if secs < 3600 {
            let mins = secs / 60;
            return format!("{} min{} ago", mins, if mins > 1 { "s" } else { "" });
        } else if secs < 86400 {
            let hours = secs / 3600;
            return format!("{} hour{} ago", hours, if hours > 1 { "s" } else { "" });
        }
    }

    // Fall back to a simple format
    "Earlier".to_string()
}

/// Render the preview panel for the selected clipboard item.
pub fn render_preview_panel(item: Option<&ClipboardItem>) -> Div {
    let t = theme();

    let panel = div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .px(t.clipboard.preview_padding)
        .py(t.clipboard.preview_padding)
        .overflow_hidden();

    let Some(item) = item else {
        return panel.child(
            div()
                .text_sm()
                .text_color(t.empty_state_color)
                .child(SharedString::from("No selection")),
        );
    };

    match &item.content {
        ClipboardContent::Text(text) => {
            // Check if this is a color string
            if let Some(color) = parse_color(text) {
                return render_color_preview(panel, &color);
            }

            // Check if this is a file:// URL
            if let Some(path) = parse_file_url(text) {
                // Treat it as a file path
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(
                        ext_lower.as_str(),
                        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg"
                    ) {
                        // Render as image
                        return panel.child(
                            img(path)
                                .w_full()
                                .h_full()
                                .object_fit(gpui::ObjectFit::Contain),
                        );
                    } else if matches!(
                        ext_lower.as_str(),
                        "txt"
                            | "md"
                            | "rs"
                            | "py"
                            | "js"
                            | "ts"
                            | "json"
                            | "yaml"
                            | "yml"
                            | "toml"
                            | "html"
                            | "css"
                            | "sh"
                    ) {
                        // Try to read and display file content
                        if let Ok(content) = fs::read_to_string(&path) {
                            let preview_content = if content.len() > 10000 {
                                format!(
                                    "{}...\n\n[Content truncated - {} bytes total]",
                                    &content[..10000],
                                    content.len()
                                )
                            } else {
                                content
                            };

                            return panel.items_start().child(
                                div()
                                    .w_full()
                                    .text_sm()
                                    .text_color(t.item_title_color)
                                    .child(SharedString::from(preview_content)),
                            );
                        }
                    }
                }
            }

            // Show full text with wrapping
            panel.items_start().child(
                div()
                    .w_full()
                    .text_sm()
                    .text_color(t.item_title_color)
                    .child(SharedString::from(text.clone())),
            )
        }
        ClipboardContent::Image {
            width,
            height,
            rgba_bytes,
        } => {
            // Try to render the image
            render_image_preview_full(panel, *width, *height, rgba_bytes)
        }
        ClipboardContent::FilePaths(paths) => {
            if paths.len() == 1 {
                let path = &paths[0];

                // Check if it's an image file
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(
                        ext_lower.as_str(),
                        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg"
                    ) {
                        // Try to load and display the image
                        return panel.child(
                            img(path.clone())
                                .w_full()
                                .h_full()
                                .object_fit(gpui::ObjectFit::Contain),
                        );
                    } else if matches!(
                        ext_lower.as_str(),
                        "txt"
                            | "md"
                            | "rs"
                            | "py"
                            | "js"
                            | "ts"
                            | "json"
                            | "yaml"
                            | "yml"
                            | "toml"
                            | "html"
                            | "css"
                            | "sh"
                    ) {
                        // Try to read and display file content
                        if let Ok(content) = fs::read_to_string(path) {
                            // Limit content size to prevent performance issues
                            let preview_content = if content.len() > 10000 {
                                format!(
                                    "{}...\n\n[Content truncated - {} bytes total]",
                                    &content[..10000],
                                    content.len()
                                )
                            } else {
                                content
                            };

                            return panel.items_start().child(
                                div()
                                    .w_full()
                                    .text_sm()
                                    .text_color(t.item_title_color)
                                    .child(SharedString::from(preview_content)),
                            );
                        }
                    }
                }

                // Fallback: show file path
                panel.items_start().child(
                    div()
                        .text_sm()
                        .text_color(t.item_description_color)
                        .child(SharedString::from(path.to_string_lossy().to_string())),
                )
            } else {
                // Multiple files: show list
                panel.items_start().child(
                    div()
                        .w_full()
                        .text_sm()
                        .text_color(t.item_title_color)
                        .child(SharedString::from(
                            paths
                                .iter()
                                .filter_map(|p| p.to_str())
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )),
                )
            }
        }
        ClipboardContent::RichText { plain, .. } => {
            // Show plain text version
            panel.items_start().child(
                div()
                    .w_full()
                    .text_sm()
                    .text_color(t.item_title_color)
                    .child(SharedString::from(plain.clone())),
            )
        }
    }
}

/// Render an image from raw RGBA bytes in the preview panel.
fn render_image_preview_full(panel: Div, width: usize, height: usize, rgba_bytes: &[u8]) -> Div {
    use image::{ImageBuffer, ImageFormat, Rgba};
    use std::io::Cursor;
    use std::sync::Arc;
    let t = theme();

    // Create ImageBuffer from raw RGBA pixel data
    if let Some(img_buffer) =
        ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, rgba_bytes.to_vec())
    {
        // Encode image as PNG in memory
        let mut png_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut png_bytes);

        if img_buffer
            .write_to(&mut cursor, ImageFormat::Png)
            .is_ok()
        {
            // Create GPUI Image from PNG bytes and wrap in Arc
            let gpui_image = Arc::new(gpui::Image::from_bytes(
                gpui::ImageFormat::Png,
                png_bytes,
            ));

            // Use the in-memory image directly
            return panel.child(
                img(gpui_image)
                    .w_full()
                    .h_full()
                    .object_fit(gpui::ObjectFit::Contain),
            );
        }
    }

    // Fallback: show error message
    panel.child(
        div()
            .text_sm()
            .text_color(t.item_description_color)
            .child(SharedString::from("[Image preview unavailable]")),
    )
}

/// Render a color preview with swatch and color codes.
fn render_color_preview(panel: Div, color: &Color) -> Div {
    let t = theme();
    let (h, s, l) = color.to_hsl();

    panel
        .flex_col()
        .items_center()
        .gap(t.clipboard.color_preview_gap)
        .child(
            // Color circle swatch
            div()
                .w(t.clipboard.color_swatch_size)
                .h(t.clipboard.color_swatch_size)
                .flex_shrink_0()
                .rounded(t.clipboard.color_swatch_size * 0.5)
                .bg(gpui::hsla(
                    h as f32 / 360.0,
                    s as f32 / 100.0,
                    l as f32 / 100.0,
                    color.a as f32 / 255.0,
                ))
                .border_1()
                .border_color(t.window_border),
        )
        .child(
            // Color codes
            div()
                .flex()
                .flex_col()
                .gap(t.clipboard.color_code_gap)
                .child(
                    // HEX
                    div()
                        .flex()
                        .flex_row()
                        .gap(t.clipboard.color_code_gap)
                        .child(
                            div()
                                .w(t.clipboard.color_label_width)
                                .text_xs()
                                .text_color(t.item_description_color)
                                .child(SharedString::from("HEX")),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(t.item_title_color)
                                .child(SharedString::from(color.to_hex())),
                        ),
                )
                .child(
                    // RGB
                    div()
                        .flex()
                        .flex_row()
                        .gap(t.clipboard.color_code_gap)
                        .child(
                            div()
                                .w(t.clipboard.color_label_width)
                                .text_xs()
                                .text_color(t.item_description_color)
                                .child(SharedString::from("RGB")),
                        )
                        .child(div().text_sm().text_color(t.item_title_color).child(
                            SharedString::from(format!("{}, {}, {}", color.r, color.g, color.b)),
                        )),
                )
                .child(
                    // HSL
                    div()
                        .flex()
                        .flex_row()
                        .gap(t.clipboard.color_code_gap)
                        .child(
                            div()
                                .w(t.clipboard.color_label_width)
                                .text_xs()
                                .text_color(t.item_description_color)
                                .child(SharedString::from("HSL")),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(t.item_title_color)
                                .child(SharedString::from(format!("{}°, {}%, {}%", h, s, l))),
                        ),
                ),
        )
}

/// Parse a file:// URL and return the path.
fn parse_file_url(text: &str) -> Option<PathBuf> {
    let text = text.trim();

    // Handle file:// URLs
    if let Some(path_str) = text.strip_prefix("file://") {
        // Remove the file:// prefix
        // URL decode the path (handle %20 for spaces, etc.)
        if let Ok(decoded) = urlencoding::decode(path_str) {
            return Some(PathBuf::from(decoded.as_ref()));
        }

        // Fallback: use the path as-is
        return Some(PathBuf::from(path_str));
    }

    None
}
