//! Rendering functions for all list item types.
//!
//! This module provides rendering logic for items_new,
//! maintaining visual equivalence with the old implementation.

use crate::assets::PhosphorIcon;
use crate::items::{DisplayItem, IconProvider, ListItem};
use crate::ui::theme::theme;
use gpui::{Div, ElementId, SharedString, Stateful, div, img, prelude::*, px, svg};
use std::path::PathBuf;

/// Render any list item based on its type.
/// This is the main dispatch function for item rendering.
pub fn render_item(item: &ListItem, selected: bool, row: usize) -> Stateful<Div> {
    match item {
        ListItem::Application(app) => render_application(app, selected, row),
        ListItem::Window(win) => render_window(win, selected, row),
        ListItem::Action(act) => render_action(act, selected, row),
        ListItem::Submenu(sub) => render_submenu(sub, selected, row),
        ListItem::Calculator(calc) => render_calculator(calc, selected, row),
        ListItem::Search(search) => render_search(search, selected, row),
        ListItem::Ai(ai) => render_ai(ai, selected, row),
        ListItem::Theme(theme) => crate::ui::views::render_theme_item(theme, selected, row),
    }
}

/// Render an application item.
fn render_application(
    app: &crate::items::ApplicationItem,
    selected: bool,
    row: usize,
) -> Stateful<Div> {
    let mut item = item_container(row, selected)
        .child(render_icon(app.icon_path.as_ref()))
        .child(render_text_content(
            &app.name,
            app.description.as_deref(),
            selected,
        ));

    if selected {
        item = item.child(render_action_indicator("Open"));
    }

    item
}

/// Render a window item.
fn render_window(win: &crate::items::WindowItem, selected: bool, row: usize) -> Stateful<Div> {
    let mut item = item_container(row, selected)
        .child(render_icon(win.icon_path.as_ref()))
        .child(render_text_content(
            &win.title,
            Some(&win.description),
            selected,
        ));

    if selected {
        item = item.child(render_action_indicator("Switch"));
    }

    item
}

/// Render an action item.
fn render_action(act: &crate::items::ActionItem, selected: bool, row: usize) -> Stateful<Div> {
    let icon = act.icon_name().and_then(PhosphorIcon::from_name);
    let mut item = item_container(row, selected)
        .child(render_phosphor_icon(icon))
        .child(render_text_content(
            &act.name,
            act.description.as_deref(),
            selected,
        ));

    if selected {
        item = item.child(render_action_indicator("Run"));
    }

    item
}

/// Render a submenu item.
fn render_submenu(sub: &crate::items::SubmenuItem, selected: bool, row: usize) -> Stateful<Div> {
    let icon = sub.icon_name().and_then(PhosphorIcon::from_name);
    let mut item = item_container(row, selected)
        .child(render_phosphor_icon(icon))
        .child(render_text_content(
            &sub.name,
            sub.description.as_deref(),
            selected,
        ));

    if selected {
        // Show arrow to indicate submenu
        item = item.child(render_action_indicator("→"));
    }

    item
}

/// Render a calculator item with special styling.
///
/// The calculator item displays:
/// - A custom "=" icon in a colored circle
/// - The expression as muted smaller text
/// - The result (or error) with "= " prefix in larger text
fn render_calculator(
    calc: &crate::items::CalculatorItem,
    selected: bool,
    row: usize,
) -> Stateful<Div> {
    let theme = theme();

    let bg_color = if selected {
        theme.item_background_selected
    } else {
        theme.item_background
    };

    let mut container = div()
        .id(ElementId::NamedInteger("calc-item".into(), row as u64))
        .mx(theme.item_margin_x)
        .my(theme.item_margin_y)
        .px(theme.item_padding_x)
        .py(theme.item_padding_y)
        .bg(bg_color)
        .rounded(theme.item_border_radius)
        .overflow_hidden()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .gap_2();

    // Add custom calculator icon
    container = container.child(render_calculator_icon());

    // Add text content
    container = container.child(render_calculator_content(calc, selected));

    // Add action indicator when selected
    if selected {
        container = container.child(render_action_indicator("Copy"));
    }

    container
}

/// Render the calculator icon (an "=" in a colored circle).
fn render_calculator_icon() -> Div {
    let theme = theme();
    let size = theme.icon_size;

    // Use theme colors for calculator icon
    let icon_bg = theme.calculator.icon_background;
    let icon_color = theme.calculator.icon_color;

    div()
        .w(size)
        .h(size)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(icon_bg)
        .rounded_sm()
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(icon_color)
                .child(SharedString::from("=")),
        )
}

/// Render the calculator text content (result only).
fn render_calculator_content(calc: &crate::items::CalculatorItem, selected: bool) -> Div {
    let theme = theme();

    let result_color = if calc.is_error {
        theme.calculator.error_color
    } else {
        theme.item_title_color
    };

    let max_width = theme.max_text_width(px(crate::config::window_width()), selected);

    div()
        .h(theme.item_content_height)
        .max_w(max_width)
        .flex()
        .flex_col()
        .justify_center()
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .text_sm()
                .line_height(theme.item_title_line_height)
                .text_color(result_color)
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child(SharedString::from(calc.display_result.clone())),
        )
}

/// Render a search item.
fn render_search(search: &crate::items::SearchItem, selected: bool, row: usize) -> Stateful<Div> {
    let mut item = item_container(row, selected)
        .child(render_phosphor_icon(Some(search.icon())))
        .child(render_text_content(&search.name, None, selected));

    if selected {
        item = item.child(render_action_indicator("Open"));
    }

    item
}

/// Render an AI item.
fn render_ai(ai: &crate::items::AiItem, selected: bool, row: usize) -> Stateful<Div> {
    let mut item = item_container(row, selected)
        .child(render_phosphor_icon(Some(ai.icon())))
        .child(render_text_content(&ai.name, ai.description(), selected));

    if selected {
        item = item.child(render_action_indicator("Ask"));
    }

    item
}

/// Create the base container for a list item with selection styling.
pub fn item_container(row: usize, selected: bool) -> Stateful<Div> {
    let theme = theme();

    let bg_color = if selected {
        theme.item_background_selected
    } else {
        theme.item_background
    };

    div()
        .id(ElementId::NamedInteger("list-item".into(), row as u64))
        .mx(theme.item_margin_x)
        .my(theme.item_margin_y)
        .px(theme.item_padding_x)
        .py(theme.item_padding_y)
        .bg(bg_color)
        .rounded(theme.item_border_radius)
        .overflow_hidden()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
}

/// Render an icon from a file path, with fallback placeholder.
pub fn render_icon(icon_path: Option<&PathBuf>) -> Div {
    let theme = theme();
    let size = theme.icon_size;

    let icon_container = div()
        .w(size)
        .h(size)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center();

    if let Some(path) = icon_path {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(ext, "png" | "jpg" | "jpeg" | "svg") {
            return icon_container.child(img(path.clone()).w(size).h(size).rounded_sm());
        }
    }

    // Fallback: show a subtle placeholder
    icon_container
        .bg(theme.icon_placeholder_background)
        .rounded_sm()
        .child(
            div()
                .text_sm()
                .text_color(theme.icon_placeholder_color)
                .child(SharedString::from("?")),
        )
}

/// Render a Phosphor icon from embedded SVG assets.
pub fn render_phosphor_icon(icon: Option<PhosphorIcon>) -> Div {
    let theme = theme();
    let size = theme.icon_size;

    let icon_container = div()
        .w(size)
        .h(size)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(theme.icon_placeholder_background)
        .rounded_sm();

    if let Some(icon) = icon {
        icon_container.child(
            svg()
                .path(icon.path())
                .size_4()
                .text_color(theme.icon_placeholder_color),
        )
    } else {
        // Fallback to placeholder
        icon_container.child(
            div()
                .text_sm()
                .text_color(theme.icon_placeholder_color)
                .child(SharedString::from("?")),
        )
    }
}

/// Render the text content (title and optional description).
pub fn render_text_content(name: &str, description: Option<&str>, selected: bool) -> Div {
    let theme = theme();

    let name_element = div()
        .w_full()
        .text_sm()
        .line_height(theme.item_title_line_height)
        .text_color(theme.item_title_color)
        .whitespace_nowrap()
        .overflow_hidden()
        .text_ellipsis()
        .child(SharedString::from(name.to_string()));

    let max_width = theme.max_text_width(px(crate::config::window_width()), selected);

    let mut content = div()
        .h(theme.item_content_height)
        .max_w(max_width)
        .flex()
        .flex_col()
        .justify_center()
        .overflow_hidden();

    content = content.child(name_element);

    if let Some(desc) = description {
        let description_element = div()
            .w_full()
            .text_xs()
            .h(theme.layout.item_description_height) // Fixed height to fit descenders
            .text_color(theme.item_description_color)
            .whitespace_nowrap()
            .overflow_hidden()
            .text_ellipsis()
            .child(SharedString::from(desc.to_string()));

        content = content.child(description_element);
    }

    content
}

/// Render the action indicator shown on selected items.
pub fn render_action_indicator(label: &str) -> Div {
    let theme = theme();

    div()
        .absolute()
        .right(theme.action_indicator.right_position)
        .top_0()
        .bottom_0()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(theme.action_indicator.label_color)
                .child(SharedString::from(label.to_string())),
        )
        .child(
            // Kbd-style box for Enter key
            div()
                .px(theme.action_indicator.key_padding_x)
                .pt(theme.action_indicator.key_padding_top)
                .pb(theme.action_indicator.key_padding_bottom)
                .bg(theme.action_indicator.key_background)
                .border_1()
                .border_color(theme.action_indicator.key_border)
                .rounded(theme.action_indicator.key_border_radius)
                .text_size(theme.action_indicator.key_font_size)
                .line_height(theme.action_indicator.key_line_height)
                .text_color(theme.action_indicator.key_color)
                .child(SharedString::from("↵")),
        )
}
