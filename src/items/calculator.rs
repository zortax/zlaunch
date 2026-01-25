use super::traits::{Categorizable, DisplayItem, Executable, IconProvider, Previewable};

/// A calculator item representing a calculation result.
#[derive(Clone, Debug)]
pub struct CalculatorItem {
    /// Unique identifier for this item.
    pub id: String,
    /// The original expression entered by the user.
    pub expression: String,
    /// The result formatted for display (with thousand separators).
    pub display_result: String,
    /// The result formatted for clipboard (raw number).
    /// None if the result is an error (NaN, Infinity).
    pub clipboard_result: Option<String>,
    /// Whether this is an error result.
    pub is_error: bool,
}

impl CalculatorItem {
    /// Get the text to copy to clipboard.
    pub fn text_for_clipboard(&self) -> &str {
        self.clipboard_result
            .as_deref()
            .unwrap_or(&self.display_result)
    }
}

impl DisplayItem for CalculatorItem {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.expression
    }

    fn description(&self) -> Option<&str> {
        Some(&self.display_result)
    }

    fn action_label(&self) -> &'static str {
        "Copy"
    }
}

impl IconProvider for CalculatorItem {
    // Calculator uses a custom icon (not path or named)
}

impl Executable for CalculatorItem {
    fn execute(&self) -> anyhow::Result<()> {
        // Copy to clipboard
        crate::clipboard::copy_to_clipboard(self.text_for_clipboard())
            .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {}", e))?;
        Ok(())
    }
}

impl Categorizable for CalculatorItem {
    fn section_name(&self) -> &'static str {
        "Calculator"
    }

    fn sort_priority(&self) -> u8 {
        0
    }
}

impl Previewable for CalculatorItem {
    fn has_preview(&self) -> bool {
        false
    }
}

impl From<CalculatorItem> for super::ListItem {
    fn from(item: CalculatorItem) -> Self {
        Self::Calculator(item)
    }
}
