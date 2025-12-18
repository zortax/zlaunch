use super::traits::{Categorizable, DisplayItem, IconProvider};
use crate::assets::PhosphorIcon;

/// An AI item representing a query to be answered by the LLM.
#[derive(Clone, Debug)]
pub struct AiItem {
    /// Unique identifier for this item
    pub id: String,
    /// Display name (e.g., "Ask Gemini")
    pub name: String,
    /// The user's query
    pub query: String,
}

impl AiItem {
    /// Create a new AI item for a query.
    pub fn new(query: String) -> Self {
        let id = format!("ai-{}", query.replace(' ', "-").to_lowercase());
        let name = "Ask AI".to_string();
        Self { id, name, query }
    }

    /// Get the icon for this AI item.
    pub fn icon(&self) -> PhosphorIcon {
        PhosphorIcon::Brain
    }
}

impl DisplayItem for AiItem {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        Some("Use an LLM to answer your question")
    }

    fn action_label(&self) -> &'static str {
        "Ask"
    }
}

impl IconProvider for AiItem {
    // Uses Phosphor icons via icon() method
}

impl Categorizable for AiItem {
    fn section_name(&self) -> &'static str {
        "AI"
    }

    fn sort_priority(&self) -> u8 {
        1
    }
}
