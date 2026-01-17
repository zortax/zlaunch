//! Theme-aware styling utilities.
//!
//! This module provides shared styling utilities that use the current theme,
//! reducing duplication across rendering components.

use crate::ui::theme::theme;
use gpui::{Hsla, Pixels, hsla};

/// Get the background color based on selection state.
///
/// Returns `item_background_selected` if selected, otherwise `item_background`.
pub fn selection_bg(selected: bool) -> Hsla {
    let t = theme();
    if selected {
        t.item_background_selected
    } else {
        t.item_background
    }
}

/// Lighten a color by adjusting its lightness.
///
/// # Arguments
/// * `color` - The color to lighten
/// * `amount` - The amount to add to the lightness (0.0 to 1.0)
///
/// # Returns
/// A new color with increased lightness, capped at 1.0
pub fn lighten_color(color: Hsla, amount: f32) -> Hsla {
    hsla(color.h, color.s, (color.l + amount).min(1.0), color.a)
}

/// Darken a color by adjusting its lightness.
///
/// # Arguments
/// * `color` - The color to darken
/// * `amount` - The amount to subtract from the lightness (0.0 to 1.0)
///
/// # Returns
/// A new color with decreased lightness, floored at 0.0
pub fn darken_color(color: Hsla, amount: f32) -> Hsla {
    hsla(color.h, color.s, (color.l - amount).max(0.0), color.a)
}

/// Get the current icon size from theme.
pub fn icon_size() -> Pixels {
    theme().icon_size
}

/// Get the current item padding from theme.
pub fn item_padding() -> (Pixels, Pixels) {
    let t = theme();
    (t.item_padding_x, t.item_padding_y)
}

/// Get the current item margin from theme.
pub fn item_margin() -> (Pixels, Pixels) {
    let t = theme();
    (t.item_margin_x, t.item_margin_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lighten_color() {
        let color = hsla(0.5, 0.5, 0.3, 1.0);
        let lightened = lighten_color(color, 0.2);
        assert!((lightened.l - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_lighten_color_caps_at_one() {
        let color = hsla(0.5, 0.5, 0.9, 1.0);
        let lightened = lighten_color(color, 0.5);
        assert!((lightened.l - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_darken_color() {
        let color = hsla(0.5, 0.5, 0.5, 1.0);
        let darkened = darken_color(color, 0.2);
        assert!((darkened.l - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_darken_color_floors_at_zero() {
        let color = hsla(0.5, 0.5, 0.1, 1.0);
        let darkened = darken_color(color, 0.5);
        assert!((darkened.l - 0.0).abs() < 0.001);
    }
}
