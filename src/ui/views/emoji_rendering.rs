//! Rendering functions for emoji grid view.

use crate::emoji::EmojiItem;
use crate::ui::theme::theme;
use gpui::{div, prelude::*, Div, ElementId, Stateful, SharedString};

/// Render a single emoji cell in the grid.
pub fn render_emoji_cell(emoji: &EmojiItem, selected: bool, index: usize) -> Stateful<Div> {
    let theme = theme();

    let bg = if selected {
        theme.emoji.cell_selected_bg
    } else {
        gpui::hsla(0.0, 0.0, 0.0, 0.0) // transparent
    };

    div()
        .id(ElementId::NamedInteger("emoji-cell".into(), index as u64))
        .w(theme.emoji.cell_size)
        .h(theme.emoji.cell_size)
        .flex()
        .items_center()
        .justify_center()
        .bg(bg)
        .rounded(theme.emoji.cell_border_radius)
        .child(
            div()
                .text_size(theme.emoji.font_size)
                .child(SharedString::from(emoji.emoji.clone())),
        )
}

/// Render a row of emoji cells.
pub fn render_emoji_row(
    emojis: &[EmojiItem],
    start_index: usize,
    selected_index: Option<usize>,
    columns: usize,
) -> Div {
    let theme = theme();

    let mut row = div()
        .w_full()
        .flex()
        .flex_row()
        .justify_center()
        .gap(theme.emoji.cell_gap);

    for (i, emoji) in emojis.iter().enumerate() {
        let global_idx = start_index + i;
        let selected = selected_index == Some(global_idx);
        row = row.child(render_emoji_cell(emoji, selected, global_idx));
    }

    // Pad with empty cells if row is not full
    let remaining = columns - emojis.len();
    for _ in 0..remaining {
        row = row.child(div().w(theme.emoji.cell_size).h(theme.emoji.cell_size));
    }

    row
}
