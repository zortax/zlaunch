use gpui::{Hsla, Pixels, hsla, px};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Custom serde module for Hsla colors
mod hsla_serde {
    use super::*;

    pub fn serialize<S>(color: &Hsla, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as an object with h, s, l, a fields
        #[derive(Serialize)]
        struct HslaHelper {
            h: f32,
            s: f32,
            l: f32,
            a: f32,
        }

        let helper = HslaHelper {
            h: color.h,
            s: color.s,
            l: color.l,
            a: color.a,
        };
        helper.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Hsla, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ColorFormat {
            // HSLA format: { h = 0.5, s = 0.8, l = 0.6, a = 1.0 }
            Hsla { h: f32, s: f32, l: f32, a: f32 },
            // RGB format with alpha: { r = 255, g = 128, b = 64, a = 128 }
            Rgba { r: u8, g: u8, b: u8, a: u8 },
            // RGB format without alpha: { r = 255, g = 128, b = 64 }
            Rgb { r: u8, g: u8, b: u8 },
            // Hex string format: "#3fc3aa" or "#3fc3aa45"
            Hex(String),
        }

        match ColorFormat::deserialize(deserializer)? {
            ColorFormat::Hsla { h, s, l, a } => Ok(hsla(h, s, l, a)),
            ColorFormat::Rgba { r, g, b, a } => Ok(rgba_to_hsla(r, g, b, a)),
            ColorFormat::Rgb { r, g, b } => Ok(rgba_to_hsla(r, g, b, 255)),
            ColorFormat::Hex(s) => parse_hex_color(&s),
        }
    }

    /// Convert RGBA (0-255) to HSLA (0-1)
    fn rgba_to_hsla(r: u8, g: u8, b: u8, a: u8) -> Hsla {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        let a = a as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        // Calculate lightness
        let l = (max + min) / 2.0;

        // Calculate saturation
        let s = if delta == 0.0 {
            0.0
        } else if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        // Calculate hue
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
        } else if max == g {
            ((b - r) / delta + 2.0) / 6.0
        } else {
            ((r - g) / delta + 4.0) / 6.0
        };

        hsla(h, s, l, a)
    }

    /// Parse hex color string (#RGB, #RGBA, #RRGGBB, or #RRGGBBAA)
    fn parse_hex_color<E: serde::de::Error>(s: &str) -> Result<Hsla, E> {
        let s = s.trim_start_matches('#');

        let (r, g, b, a) = match s.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = u8::from_str_radix(&s[0..1].repeat(2), 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let g = u8::from_str_radix(&s[1..2].repeat(2), 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let b = u8::from_str_radix(&s[2..3].repeat(2), 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                (r, g, b, 255)
            }
            4 => {
                // #RGBA -> #RRGGBBAA
                let r = u8::from_str_radix(&s[0..1].repeat(2), 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let g = u8::from_str_radix(&s[1..2].repeat(2), 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let b = u8::from_str_radix(&s[2..3].repeat(2), 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let a = u8::from_str_radix(&s[3..4].repeat(2), 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                (r, g, b, a)
            }
            6 => {
                // #RRGGBB
                let r = u8::from_str_radix(&s[0..2], 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let g = u8::from_str_radix(&s[2..4], 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let b = u8::from_str_radix(&s[4..6], 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                (r, g, b, 255)
            }
            8 => {
                // #RRGGBBAA
                let r = u8::from_str_radix(&s[0..2], 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let g = u8::from_str_radix(&s[2..4], 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let b = u8::from_str_radix(&s[4..6], 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                let a = u8::from_str_radix(&s[6..8], 16)
                    .map_err(|e| E::custom(format!("Invalid hex color: {}", e)))?;
                (r, g, b, a)
            }
            _ => return Err(E::custom(format!("Invalid hex color length: {}", s.len()))),
        };

        Ok(rgba_to_hsla(r, g, b, a))
    }
}

/// Custom serde module for Pixels
mod pixels_serde {
    use super::*;

    pub fn serialize<S>(pixels: &Pixels, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize using format!() since inner field is private
        let s = format!("{:?}", pixels);
        // Extract the number from "Pixels(123.0)" format
        let num_str = s.trim_start_matches("Pixels(").trim_end_matches(')');
        if let Ok(value) = num_str.parse::<f32>() {
            serializer.serialize_f32(value)
        } else {
            // Fallback: just serialize as string
            serializer.serialize_str(&s)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pixels, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Try to deserialize as f32 first
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PixelsHelper {
            Float(f32),
            String(String),
        }

        match PixelsHelper::deserialize(deserializer)? {
            PixelsHelper::Float(value) => Ok(px(value)),
            PixelsHelper::String(s) => {
                // Parse from "Pixels(123.0)" format
                let num_str = s.trim_start_matches("Pixels(").trim_end_matches(')');
                num_str
                    .parse::<f32>()
                    .map(px)
                    .map_err(|e| serde::de::Error::custom(format!("Invalid pixels value: {}", e)))
            }
        }
    }
}

/// Calculator-specific styling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CalculatorTheme {
    /// Background color for the calculator icon
    #[serde(with = "hsla_serde")]
    pub icon_background: Hsla,
    /// Foreground color for the calculator icon
    #[serde(with = "hsla_serde")]
    pub icon_color: Hsla,
    /// Color for calculator error messages
    #[serde(with = "hsla_serde")]
    pub error_color: Hsla,
}

/// Action indicator styling (shown on the right side of selected items).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ActionIndicatorTheme {
    /// Total width allocated for the action indicator
    #[serde(with = "pixels_serde")]
    pub width: Pixels,
    /// Right margin positioning
    #[serde(with = "pixels_serde")]
    pub right_position: Pixels,
    /// Color for the action label text
    #[serde(with = "hsla_serde")]
    pub label_color: Hsla,
    /// Background color for key badges
    #[serde(with = "hsla_serde")]
    pub key_background: Hsla,
    /// Border color for key badges
    #[serde(with = "hsla_serde")]
    pub key_border: Hsla,
    /// Text color for key badges
    #[serde(with = "hsla_serde")]
    pub key_color: Hsla,
    /// Horizontal padding for key badges
    #[serde(with = "pixels_serde")]
    pub key_padding_x: Pixels,
    /// Top padding for key badges
    #[serde(with = "pixels_serde")]
    pub key_padding_top: Pixels,
    /// Bottom padding for key badges
    #[serde(with = "pixels_serde")]
    pub key_padding_bottom: Pixels,
    /// Border radius for key badges
    #[serde(with = "pixels_serde")]
    pub key_border_radius: Pixels,
    /// Font size for key badge text
    #[serde(with = "pixels_serde")]
    pub key_font_size: Pixels,
    /// Line height for key badge text
    #[serde(with = "pixels_serde")]
    pub key_line_height: Pixels,
}

/// Emoji picker grid styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmojiTheme {
    /// Number of columns in the emoji grid
    pub columns: usize,
    /// Size of each emoji cell (width and height)
    #[serde(with = "pixels_serde")]
    pub cell_size: Pixels,
    /// Font size for emoji characters
    #[serde(with = "pixels_serde")]
    pub font_size: Pixels,
    /// Background color for selected emoji cells
    #[serde(with = "hsla_serde")]
    pub cell_selected_bg: Hsla,
    /// Border radius for emoji cells
    #[serde(with = "pixels_serde")]
    pub cell_border_radius: Pixels,
    /// Gap between emoji cells
    #[serde(with = "pixels_serde")]
    pub cell_gap: Pixels,
}

/// AI response view styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiTheme {
    /// Background color for error state
    #[serde(with = "hsla_serde")]
    pub error_background: Hsla,
    /// Text color for error titles
    #[serde(with = "hsla_serde")]
    pub error_title_color: Hsla,
    /// Text color for error messages
    #[serde(with = "hsla_serde")]
    pub error_message_color: Hsla,
}

/// Markdown rendering styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarkdownTheme {
    /// Line height for paragraph text
    #[serde(with = "pixels_serde")]
    pub paragraph_line_height: Pixels,
    /// Line height for heading text
    #[serde(with = "pixels_serde")]
    pub heading_line_height: Pixels,
    /// Border radius for code blocks
    #[serde(with = "pixels_serde")]
    pub code_block_radius: Pixels,
    /// Font family for code blocks
    #[serde(skip)]
    pub code_font_family: &'static str,
    /// Line height for code text
    #[serde(with = "pixels_serde")]
    pub code_line_height: Pixels,
}

/// Clipboard preview panel styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipboardTheme {
    /// Size of color preview icons in the list
    #[serde(with = "pixels_serde")]
    pub color_icon_size: Pixels,
    /// Padding for the preview panel
    #[serde(with = "pixels_serde")]
    pub preview_padding: Pixels,
    /// Size of the large color swatch in preview
    #[serde(with = "pixels_serde")]
    pub color_swatch_size: Pixels,
    /// Gap between color preview elements
    #[serde(with = "pixels_serde")]
    pub color_preview_gap: Pixels,
    /// Gap between color code elements
    #[serde(with = "pixels_serde")]
    pub color_code_gap: Pixels,
    /// Width for color code labels
    #[serde(with = "pixels_serde")]
    pub color_label_width: Pixels,
}

/// Section header styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SectionHeaderTheme {
    /// Text color for section headers
    #[serde(with = "hsla_serde")]
    pub color: Hsla,
    /// Top margin for section headers
    #[serde(with = "pixels_serde")]
    pub margin_top: Pixels,
    /// Bottom margin for section headers
    #[serde(with = "pixels_serde")]
    pub margin_bottom: Pixels,
    /// Vertical padding for section headers
    #[serde(with = "pixels_serde")]
    pub padding_y: Pixels,
}

/// General layout values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutTheme {
    /// Width of separator lines
    #[serde(with = "pixels_serde")]
    pub separator_width: Pixels,
    /// Fixed height for item description text (to fit descenders)
    #[serde(with = "pixels_serde")]
    pub item_description_height: Pixels,
}

/// Centralized theme configuration for the launcher UI.
/// All colors, sizes, and spacing are defined here for consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherTheme {
    /// Theme name identifier
    pub name: String,

    // Window colors and styling (size is configured in AppConfig)
    #[serde(with = "hsla_serde")]
    pub window_background: Hsla,
    #[serde(with = "hsla_serde")]
    pub window_border: Hsla,
    #[serde(with = "pixels_serde")]
    pub window_border_radius: Pixels,

    // List items
    #[serde(with = "pixels_serde")]
    pub item_margin_x: Pixels,
    #[serde(with = "pixels_serde")]
    pub item_margin_y: Pixels,
    #[serde(with = "pixels_serde")]
    pub item_padding_x: Pixels,
    #[serde(with = "pixels_serde")]
    pub item_padding_y: Pixels,
    #[serde(with = "pixels_serde")]
    pub item_border_radius: Pixels,
    #[serde(with = "hsla_serde")]
    pub item_background: Hsla,
    #[serde(with = "hsla_serde")]
    pub item_background_selected: Hsla,

    // Item content
    #[serde(with = "hsla_serde")]
    pub item_title_color: Hsla,
    #[serde(with = "hsla_serde")]
    pub item_description_color: Hsla,
    #[serde(with = "pixels_serde")]
    pub item_title_line_height: Pixels,
    #[serde(with = "pixels_serde")]
    pub item_content_height: Pixels,

    // Icons
    #[serde(with = "pixels_serde")]
    pub icon_size: Pixels,
    #[serde(with = "hsla_serde")]
    pub icon_placeholder_background: Hsla,
    #[serde(with = "hsla_serde")]
    pub icon_placeholder_color: Hsla,

    // Empty state
    #[serde(with = "pixels_serde")]
    pub empty_state_height: Pixels,
    #[serde(with = "hsla_serde")]
    pub empty_state_color: Hsla,

    // Sub-themes for specific components
    pub calculator: CalculatorTheme,
    pub action_indicator: ActionIndicatorTheme,
    pub emoji: EmojiTheme,
    pub ai: AiTheme,
    pub markdown: MarkdownTheme,
    pub clipboard: ClipboardTheme,
    pub section_header: SectionHeaderTheme,
    pub layout: LayoutTheme,
}

impl Default for CalculatorTheme {
    fn default() -> Self {
        Self {
            icon_background: hsla(210.0 / 360.0, 0.6, 0.5, 0.15),
            icon_color: hsla(210.0 / 360.0, 0.7, 0.7, 1.0),
            error_color: hsla(15.0 / 360.0, 0.7, 0.6, 1.0),
        }
    }
}

impl Default for ActionIndicatorTheme {
    fn default() -> Self {
        Self {
            width: px(64.0),
            right_position: px(8.0),
            label_color: hsla(0.0, 0.0, 1.0, 0.4),
            key_background: hsla(0.0, 0.0, 1.0, 0.06),
            key_border: hsla(0.0, 0.0, 1.0, 0.12),
            key_color: hsla(0.0, 0.0, 1.0, 0.6),
            key_padding_x: px(4.0),
            key_padding_top: px(2.0),
            key_padding_bottom: px(1.0),
            key_border_radius: px(3.0),
            key_font_size: px(10.0),
            key_line_height: px(10.0),
        }
    }
}

impl Default for EmojiTheme {
    fn default() -> Self {
        Self {
            columns: 8,
            cell_size: px(64.0),
            font_size: px(28.0),
            cell_selected_bg: hsla(0.0, 0.0, 1.0, 0.1),
            cell_border_radius: px(6.0),
            cell_gap: px(2.0),
        }
    }
}

impl Default for AiTheme {
    fn default() -> Self {
        Self {
            error_background: hsla(0.0, 0.7, 0.3, 1.0),
            error_title_color: hsla(0.0, 1.0, 0.7, 1.0),
            error_message_color: hsla(0.0, 1.0, 0.83, 1.0),
        }
    }
}

impl Default for MarkdownTheme {
    fn default() -> Self {
        Self {
            paragraph_line_height: px(20.0),
            heading_line_height: px(22.0),
            code_block_radius: px(6.0),
            code_font_family: "Hack Nerd Font Mono",
            code_line_height: px(18.0),
        }
    }
}

impl Default for ClipboardTheme {
    fn default() -> Self {
        Self {
            color_icon_size: px(16.0),
            preview_padding: px(16.0),
            color_swatch_size: px(120.0),
            color_preview_gap: px(20.0),
            color_code_gap: px(8.0),
            color_label_width: px(60.0),
        }
    }
}

impl Default for SectionHeaderTheme {
    fn default() -> Self {
        Self {
            color: hsla(0.0, 0.0, 1.0, 0.4),
            margin_top: px(8.0),
            margin_bottom: px(4.0),
            padding_y: px(6.0),
        }
    }
}

impl Default for LayoutTheme {
    fn default() -> Self {
        Self {
            separator_width: px(1.0),
            item_description_height: px(18.0),
        }
    }
}

impl Default for LauncherTheme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),

            // Window colors and styling
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

            // Empty state
            empty_state_height: px(200.0),
            empty_state_color: hsla(0.0, 0.0, 1.0, 0.25), // 25% white

            // Sub-themes use their Default implementations
            calculator: CalculatorTheme::default(),
            action_indicator: ActionIndicatorTheme::default(),
            emoji: EmojiTheme::default(),
            ai: AiTheme::default(),
            markdown: MarkdownTheme::default(),
            clipboard: ClipboardTheme::default(),
            section_header: SectionHeaderTheme::default(),
            layout: LayoutTheme::default(),
        }
    }
}

impl LauncherTheme {
    /// Calculate the maximum text width for item content.
    /// Accounts for window width, margins, padding, icon, and optionally action indicator.
    pub fn max_text_width(&self, window_width: Pixels, with_action_indicator: bool) -> Pixels {
        let base = window_width
            - self.item_margin_x * 2.0
            - self.item_padding_x * 2.0
            - self.icon_size
            - px(8.0)  // gap between icon and text
            - px(16.0); // buffer

        if with_action_indicator {
            base - self.action_indicator.width
        } else {
            base
        }
    }
}

/// Global theme instance (cached for performance, synced from config).
static THEME: std::sync::RwLock<Option<LauncherTheme>> = std::sync::RwLock::new(None);

/// Get the global launcher theme.
/// Returns the cached theme, or loads from config on first access.
pub fn theme() -> LauncherTheme {
    let read_lock = THEME.read().unwrap();
    if let Some(theme) = read_lock.as_ref() {
        return theme.clone();
    }
    drop(read_lock);

    // Initialize theme from config
    let loaded_theme = crate::config::load_configured_theme();
    let mut write_lock = THEME.write().unwrap();
    *write_lock = Some(loaded_theme.clone());
    loaded_theme
}

/// Update the global theme (used for live preview during theme picker).
/// This does NOT persist to config - use config::update_config() for that.
pub fn set_theme(new_theme: LauncherTheme) {
    let mut write_lock = THEME.write().unwrap();
    *write_lock = Some(new_theme);
}

/// Sync the theme cache from config.
/// Call this after updating config.theme to refresh the cached theme.
pub fn sync_theme_from_config() {
    let loaded_theme = crate::config::load_configured_theme();
    let mut write_lock = THEME.write().unwrap();
    *write_lock = Some(loaded_theme);
}
