/// Represents a color with RGBA components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color from RGB components (alpha defaults to 255)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color from RGBA components
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Convert to HSL format (hue: 0-360, saturation: 0-100, lightness: 0-100)
    pub fn to_hsl(&self) -> (u16, u8, u8) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta == 0.0 {
            return (0, 0, (l * 100.0) as u8);
        }

        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let h = if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };

        (h as u16, (s * 100.0) as u8, (l * 100.0) as u8)
    }

    /// Convert to RGB format string
    pub fn to_rgb_string(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// Convert to RGBA format string
    pub fn to_rgba_string(&self) -> String {
        format!(
            "rgba({}, {}, {}, {})",
            self.r,
            self.g,
            self.b,
            self.a as f32 / 255.0
        )
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// Convert to HSL format string
    pub fn to_hsl_string(&self) -> String {
        let (h, s, l) = self.to_hsl();
        format!("hsl({}, {}%, {}%)", h, s, l)
    }
}

/// Try to parse a color string (hex, rgb, rgba, hsl, etc.)
pub fn parse_color(text: &str) -> Option<Color> {
    let text = text.trim();

    // Try hex format: #RGB, #RRGGBB, #RRGGBBAA
    if let Some(hex) = text.strip_prefix('#') {
        if hex.len() == 3 {
            // #RGB -> #RRGGBB
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            return Some(Color { r, g, b, a: 255 });
        } else if hex.len() == 6 {
            // #RRGGBB
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color { r, g, b, a: 255 });
        } else if hex.len() == 8 {
            // #RRGGBBAA
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            return Some(Color { r, g, b, a });
        }
    }

    // Try rgb/rgba format: rgb(r, g, b) or rgba(r, g, b, a)
    if text.starts_with("rgb(") || text.starts_with("rgba(") {
        let start = if text.starts_with("rgba(") { 5 } else { 4 };
        let end = text.rfind(')')?;
        let values = &text[start..end];

        let parts: Vec<&str> = values.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            let a = if parts.len() >= 4 {
                (parts[3].parse::<f32>().ok()? * 255.0) as u8
            } else {
                255
            };
            return Some(Color { r, g, b, a });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_3() {
        let color = parse_color("#f0a").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 170);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_parse_hex_6() {
        let color = parse_color("#ff00aa").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 170);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_parse_rgb() {
        let color = parse_color("rgb(255, 128, 64)").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_to_hex() {
        let color = Color::from_rgb(255, 128, 64);
        assert_eq!(color.to_hex(), "#FF8040");
    }
}
