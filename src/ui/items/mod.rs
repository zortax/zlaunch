mod application;
mod base;
mod delegate;

pub use application::render_application;
pub use base::{item_container, render_action_indicator, render_icon, render_text_content};
pub use delegate::ItemListDelegate;

use crate::items::ListItem;
use gpui::{Div, Stateful, prelude::*};

/// Render any list item based on its type.
/// This is the main dispatch function for item rendering.
pub fn render_item(item: &ListItem, selected: bool, row: usize) -> Stateful<Div> {
    match item {
        ListItem::Application(app) => render_application(app, selected, row),
        ListItem::Window(win) => render_window(win, selected, row),
        ListItem::Action(act) => render_action(act, selected, row),
        ListItem::Submenu(sub) => render_submenu(sub, selected, row),
    }
}

// Placeholder renderers for future item types

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

fn render_action(act: &crate::items::ActionItem, selected: bool, row: usize) -> Stateful<Div> {
    // Actions don't have file-based icons yet, use placeholder
    let mut item = item_container(row, selected)
        .child(render_icon(None))
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

fn render_submenu(sub: &crate::items::SubmenuItem, selected: bool, row: usize) -> Stateful<Div> {
    // Submenus don't have file-based icons yet, use placeholder
    let mut item = item_container(row, selected)
        .child(render_icon(None))
        .child(render_text_content(
            &sub.name,
            sub.description.as_deref(),
            selected,
        ));

    if selected {
        // Show arrow instead of "Open" to indicate submenu
        item = item.child(render_action_indicator("→"));
    }

    item
}
