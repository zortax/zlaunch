use gpui::{Hsla, Pixels, hsla, px};

/// Centralized theme configuration for the launcher UI.
/// All colors, sizes, and spacing are defined here for consistency.
pub struct LauncherTheme {
    // Window
    pub window_width: Pixels,
    pub window_height: Pixels,
    pub window_background: Hsla,
    pub window_border: Hsla,
    pub window_border_radius: Pixels,

    // List items
    pub item_margin_x: Pixels,
    pub item_margin_y: Pixels,
    pub item_padding_x: Pixels,
    pub item_padding_y: Pixels,
    pub item_border_radius: Pixels,
    pub item_background: Hsla,
    pub item_background_selected: Hsla,

    // Item content
    pub item_title_color: Hsla,
    pub item_description_color: Hsla,
    pub item_title_line_height: Pixels,
    pub item_content_height: Pixels,

    // Icons
    pub icon_size: Pixels,
    pub icon_placeholder_background: Hsla,
    pub icon_placeholder_color: Hsla,

    // Action indicator
    pub action_indicator_width: Pixels,
    pub action_label_color: Hsla,
    pub action_key_background: Hsla,
    pub action_key_border: Hsla,
    pub action_key_color: Hsla,

    // Empty state
    pub empty_state_height: Pixels,
    pub empty_state_color: Hsla,

    // Section headers
    pub section_header_color: Hsla,
    pub section_header_margin_top: Pixels,
    pub section_header_margin_bottom: Pixels,
}

impl Default for LauncherTheme {
    fn default() -> Self {
        Self {
            // Window - centered overlay
            window_width: px(600.0),
            window_height: px(400.0),
            window_background: hsla(0.0, 0.0, 0.06, 0.7), // ~70% opaque dark
            window_border: hsla(0.0, 0.0, 1.0, 0.094),    // ~9% white
            window_border_radius: px(12.0),

            // List items
            item_margin_x: px(8.0),
            item_margin_y: px(1.0),
            item_padding_x: px(8.0),
            item_padding_y: px(7.0),
            item_border_radius: px(6.0),
            item_background: hsla(0.0, 0.0, 0.0, 0.0), // transparent
            item_background_selected: hsla(0.0, 0.0, 1.0, 0.07), // ~7% white

            // Item content
            item_title_color: hsla(0.0, 0.0, 1.0, 0.9), // 90% white
            item_description_color: hsla(0.0, 0.0, 1.0, 0.4), // 40% white
            item_title_line_height: px(16.0),
            item_content_height: px(34.0),

            // Icons
            icon_size: px(24.0),
            icon_placeholder_background: hsla(0.0, 0.0, 1.0, 0.04), // ~4% white
            icon_placeholder_color: hsla(0.0, 0.0, 1.0, 0.25),      // 25% white

            // Action indicator (shown on selected items)
            action_indicator_width: px(64.0),
            action_label_color: hsla(0.0, 0.0, 1.0, 0.4), // 40% white
            action_key_background: hsla(0.0, 0.0, 1.0, 0.06), // ~6% white
            action_key_border: hsla(0.0, 0.0, 1.0, 0.12), // ~12% white
            action_key_color: hsla(0.0, 0.0, 1.0, 0.6),   // 60% white

            // Empty state
            empty_state_height: px(200.0),
            empty_state_color: hsla(0.0, 0.0, 1.0, 0.25), // 25% white

            // Section headers
            section_header_color: hsla(0.0, 0.0, 1.0, 0.4), // 40% white (like descriptions)
            section_header_margin_top: px(8.0),
            section_header_margin_bottom: px(4.0),
        }
    }
}

impl LauncherTheme {
    /// Calculate the maximum text width for item content.
    /// Accounts for window width, margins, padding, icon, and optionally action indicator.
    pub fn max_text_width(&self, with_action_indicator: bool) -> Pixels {
        let base = self.window_width
            - self.item_margin_x * 2.0
            - self.item_padding_x * 2.0
            - self.icon_size
            - px(8.0)  // gap between icon and text
            - px(16.0); // buffer

        if with_action_indicator {
            base - self.action_indicator_width
        } else {
            base
        }
    }
}

/// Global theme instance.
/// In the future, this could support multiple themes or runtime switching.
static THEME: std::sync::OnceLock<LauncherTheme> = std::sync::OnceLock::new();

/// Get the global launcher theme.
pub fn theme() -> &'static LauncherTheme {
    THEME.get_or_init(LauncherTheme::default)
}
