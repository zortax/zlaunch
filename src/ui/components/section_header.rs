use crate::ui::theme::theme;
use gpui::{div, prelude::*, Div, SharedString};

/// A section header component for dividing groups of items in a list.
///
/// # Example
/// ```ignore
/// SectionHeader::new("Applications")
///     .with_count(42)
///     .render()
/// ```
pub struct SectionHeader {
    title: String,
    count: Option<usize>,
}

impl SectionHeader {
    /// Create a new section header
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            count: None,
        }
    }

    /// Set the item count to display
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Render the section header
    pub fn render(self) -> Div {
        let theme = theme();

        let mut header = div()
            .px(theme.item_padding_x)
            .py(theme.section_header.padding_y)
            .flex()
            .flex_row()
            .items_center()
            .gap_2();

        // Section title
        let mut title_text = self.title.clone();
        if let Some(count) = self.count {
            title_text.push_str(&format!(" ({})", count));
        }

        header = header.child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.section_header.color)
                .child(SharedString::from(title_text.to_uppercase())),
        );

        header
    }
}
