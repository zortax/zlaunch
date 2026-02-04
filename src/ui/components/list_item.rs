use crate::ui::theme::theme;
use gpui::{Div, ElementId, SharedString, Stateful, div, img, prelude::*, px};
use std::path::PathBuf;
use std::sync::Arc;

/// A standard list item component with icon, title, description, and action indicator.
///
/// This consolidates the common list item rendering pattern used across all views.
/// Use the builder pattern to customize the appearance.
///
/// # Example
/// ```ignore
/// ListItemComponent::new(row, selected)
///     .with_icon(Some(&icon_path))
///     .with_title("Application Name")
///     .with_description(Some("Description text"))
///     .with_action_label("Open")
///     .render()
/// ```
pub struct ListItemComponent {
    row: usize,
    selected: bool,
    icon: Option<Icon>,
    title: String,
    description: Option<String>,
    action_label: Option<String>,
}

/// Icon source for list items
pub enum Icon {
    /// Path to an image file
    Path(PathBuf),
    /// In-memory PNG image data
    Data(Arc<gpui::Image>),
    /// Named Phosphor icon
    Named(String),
    /// Custom placeholder text
    Placeholder(String),
}

impl Icon {
    /// Create an icon from PNG bytes
    pub fn from_png_bytes(bytes: Vec<u8>) -> Self {
        let image = Arc::new(gpui::Image::from_bytes(gpui::ImageFormat::Png, bytes));
        Icon::Data(image)
    }
}

impl ListItemComponent {
    /// Create a new list item component
    pub fn new(row: usize, selected: bool) -> Self {
        Self {
            row,
            selected,
            icon: None,
            title: String::new(),
            description: None,
            action_label: None,
        }
    }

    /// Set the icon from a path
    pub fn with_icon_path(mut self, path: Option<&PathBuf>) -> Self {
        self.icon = path.map(|p| Icon::Path(p.clone()));
        self
    }

    /// Set a named icon
    pub fn with_icon_name(mut self, name: &str) -> Self {
        self.icon = Some(Icon::Named(name.to_string()));
        self
    }

    /// Set a placeholder icon
    pub fn with_placeholder(mut self, text: &str) -> Self {
        self.icon = Some(Icon::Placeholder(text.to_string()));
        self
    }

    /// Set the title text
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the description text
    pub fn with_description(mut self, description: Option<&str>) -> Self {
        self.description = description.map(|s| s.to_string());
        self
    }

    /// Set the action label (shown when selected)
    pub fn with_action_label(mut self, label: &str) -> Self {
        self.action_label = Some(label.to_string());
        self
    }

    /// Render the list item
    pub fn render(self) -> Stateful<Div> {
        let theme = theme();

        let bg_color = if self.selected {
            theme.item_background_selected
        } else {
            theme.item_background
        };

        let mut container = div()
            .id(ElementId::NamedInteger("list-item".into(), self.row as u64))
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

        // Add icon if present
        if let Some(icon) = self.icon {
            container = container.child(render_icon_element(icon));
        }

        // Add text content
        container = container.child(render_text_content(
            &self.title,
            self.description.as_deref(),
            self.selected,
        ));

        // Add action indicator if selected and label provided
        if self.selected
            && let Some(label) = self.action_label
        {
            container = container.child(render_action_indicator(&label));
        }

        container
    }
}

/// Render an icon element based on the icon type
fn render_icon_element(icon: Icon) -> Div {
    let theme = theme();
    let size = theme.icon_size;

    let icon_container = div()
        .w(size)
        .h(size)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center();

    match icon {
        Icon::Path(path) => {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext, "png" | "jpg" | "jpeg" | "svg") {
                icon_container.child(img(path).w(size).h(size).rounded_sm())
            } else {
                render_placeholder_icon(icon_container, "?")
            }
        }
        Icon::Data(image) => icon_container.child(img(image).w(size).h(size).rounded_sm()),
        Icon::Named(_name) => {
            // For named icons, we'd typically use an icon library
            // For now, use placeholder
            render_placeholder_icon(icon_container, "⚡")
        }
        Icon::Placeholder(text) => render_placeholder_icon(icon_container, &text),
    }
}

/// Render a placeholder icon
fn render_placeholder_icon(container: Div, text: &str) -> Div {
    let theme = theme();
    container
        .bg(theme.icon_placeholder_background)
        .rounded_sm()
        .child(
            div()
                .text_sm()
                .text_color(theme.icon_placeholder_color)
                .child(SharedString::from(text.to_string())),
        )
}

/// Render the text content (title and optional description)
fn render_text_content(name: &str, description: Option<&str>, selected: bool) -> Div {
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

    let max_width = theme.max_text_width(px(crate::config::launcher_size().0), selected);

    let mut content = div()
        .h(theme.item_content_height)
        .max_w(max_width)
        .flex()
        .flex_col()
        .justify_center()
        .overflow_hidden()
        .child(name_element);

    if let Some(desc) = description {
        let description_element = div()
            .w_full()
            .text_xs()
            .h(theme.layout.item_description_height)
            .text_color(theme.item_description_color)
            .whitespace_nowrap()
            .overflow_hidden()
            .text_ellipsis()
            .child(SharedString::from(desc.to_string()));

        content = content.child(description_element);
    }

    content
}

/// Render the action indicator shown on selected items
fn render_action_indicator(label: &str) -> Div {
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
