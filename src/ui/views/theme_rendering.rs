use crate::items::ThemeItem;
use crate::ui::styled::lighten_color;
use crate::ui::theme::{LauncherTheme, theme};
use crate::ui::views::{item_container, render_action_indicator, render_text_content};
use gpui::{Div, Stateful, div, prelude::*, px};

/// Render a theme item with a dynamic color preview icon.
pub fn render_theme_item(theme_item: &ThemeItem, selected: bool, row: usize) -> Stateful<Div> {
    let mut item = item_container(row, selected)
        .child(render_theme_icon(&theme_item.theme))
        .child(render_text_content(
            &theme_item.name,
            Some(theme_item.description.as_str()),
            selected,
        ));

    if selected {
        item = item.child(render_action_indicator("Apply"));
    }

    item
}

/// Render a dynamic theme preview icon showing the theme's color scheme.
///
/// The icon consists of:
/// - A rounded square background using the theme's window background color
/// - Small colored shapes inside showing other dominant theme colors:
///   - Top-left: item title color (text)
///   - Top-right: item selected background color
///   - Bottom: window border color (accent)
///
/// This gives users an immediate visual indication of the theme's color scheme.
pub fn render_theme_icon(theme_item: &LauncherTheme) -> Div {
    let current_theme = theme();
    let icon_size = current_theme.icon_size;

    // Scale factors for the internal elements
    let inner_size = icon_size * 0.35;
    let inner_gap = icon_size * 0.08;
    let inner_padding = icon_size * 0.15;
    let inner_radius = icon_size * 0.12;

    div()
        .w(icon_size)
        .h(icon_size)
        .flex_shrink_0()
        .flex()
        .items_start()
        .justify_start()
        .bg(lighten_color(theme_item.window_background, 0.05))
        .rounded(px(6.0))
        .p(inner_padding)
        .child(
            div()
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .gap(inner_gap)
                // Top row with two colored squares
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .gap(inner_gap)
                        .child(
                            // Title color square
                            div()
                                .w(inner_size)
                                .h(inner_size)
                                .bg(theme_item.item_title_color)
                                .rounded(inner_radius),
                        )
                        .child(
                            // Selected background square
                            div()
                                .w(inner_size)
                                .h(inner_size)
                                .bg(lighten_color(theme_item.item_background_selected, 0.1))
                                .rounded(inner_radius),
                        ),
                )
                // Bottom accent bar
                .child(
                    div()
                        .w_full()
                        .flex_1()
                        .bg(theme_item.window_border)
                        .rounded(inner_radius),
                ),
        )
}

