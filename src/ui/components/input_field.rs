use crate::ui::theme::theme;
use gpui_kit::component::input::Input;
use gpui_kit::{Div, div, prelude::*, px};

/// A styled input field component for search/query input.
///
/// This wraps the gpui_kit Input with consistent styling.
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
