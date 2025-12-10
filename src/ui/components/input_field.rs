use crate::ui::theme::theme;
use gpui::{div, prelude::*, px, Div};
use gpui_component::input::Input;

/// A styled input field component for search/query input.
///
/// This wraps the gpui_component Input with consistent styling.
pub struct InputField;

impl InputField {
    /// Render an input field with the given Input component
    pub fn render(input: Input) -> Div {
        let theme = theme();

        div()
            .w_full()
            .px(theme.item_padding_x)
            .py(px(12.0))
            .child(input)
    }
}
