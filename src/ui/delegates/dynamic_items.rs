//! Dynamic item detection for calculator, AI, and search.
//!
//! These items are generated on-the-fly based on the user's query,
//! rather than being static items in the list.

use crate::calculator::evaluate_expression;
use crate::items::{AiItem, CalculatorItem, SearchItem};
use crate::search::{detect_search, get_providers, SearchDetection};

/// Container for dynamically generated items based on user query.
#[derive(Clone, Default)]
pub struct DynamicItems {
    /// Calculator result (shown at top when query is a math expression).
    pub calculator_item: Option<CalculatorItem>,
    /// AI query item (shown when query triggers AI).
    pub ai_item: Option<AiItem>,
    /// Search provider items (shown when query triggers search).
    pub search_items: Vec<SearchItem>,
}

impl DynamicItems {
    /// Create a new empty dynamic items container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a query and detect dynamic items.
    ///
    /// # Arguments
    /// * `query` - The user's search query
    /// * `calculator_enabled` - Whether calculator module is enabled
    /// * `ai_enabled` - Whether AI module is enabled and configured
    /// * `search_enabled` - Whether search module is enabled
    pub fn process_query(
        &mut self,
        query: &str,
        calculator_enabled: bool,
        ai_enabled: bool,
        search_enabled: bool,
    ) {
        // Clear previous items
        self.clear();

        let trimmed = query.trim();
        if trimmed.is_empty() {
            return;
        }

        // Check for calculator expression
        if calculator_enabled && query.chars().any(|c| c.is_numeric()) {
            if let Ok(result) = evaluate_expression(query) {
                self.calculator_item = Some(result);
            }
        }

        // Check for trigger phrases
        let has_ai_trigger = trimmed.starts_with("!ai");
        let search_detection = detect_search(query);
        let has_search_trigger = matches!(search_detection, SearchDetection::Triggered { .. });

        // Logic:
        // 1. If !ai trigger → only show AI item
        // 2. Else if search trigger (!g, !ddg, etc.) → only show that search provider
        // 3. Else if query not empty → show AI item + all search providers at bottom

        if ai_enabled && has_ai_trigger {
            // Only show AI item when !ai trigger is used
            let ai_query = trimmed.strip_prefix("!ai").unwrap().trim();
            if !ai_query.is_empty() {
                self.ai_item = Some(AiItem::new(ai_query.to_string()));
            }
        } else if search_enabled && has_search_trigger {
            // Only show the triggered search provider
            if let SearchDetection::Triggered { provider, query } = search_detection {
                self.search_items.push(SearchItem::new(provider, query));
            }
        } else {
            // Show AI item and all search providers when query is not empty
            if ai_enabled {
                self.ai_item = Some(AiItem::new(trimmed.to_string()));
            }
            if search_enabled {
                if let SearchDetection::Fallback { query } = search_detection {
                    for provider in get_providers() {
                        self.search_items.push(SearchItem::new(provider, query.clone()));
                    }
                }
            }
        }
    }

    /// Clear all dynamic items.
    pub fn clear(&mut self) {
        self.calculator_item = None;
        self.ai_item = None;
        self.search_items.clear();
    }

    /// Get the total count of dynamic items.
    pub fn count(&self) -> usize {
        let calc_count = if self.calculator_item.is_some() { 1 } else { 0 };
        let ai_count = if self.ai_item.is_some() { 1 } else { 0 };
        calc_count + ai_count + self.search_items.len()
    }

    /// Check if there's a calculator item.
    pub fn has_calculator(&self) -> bool {
        self.calculator_item.is_some()
    }

    /// Check if there's an AI item.
    pub fn has_ai(&self) -> bool {
        self.ai_item.is_some()
    }

    /// Get the search items count.
    pub fn search_count(&self) -> usize {
        self.search_items.len()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let items = DynamicItems::new();
        assert_eq!(items.count(), 0);
        assert!(!items.has_calculator());
        assert!(!items.has_ai());
        assert_eq!(items.search_count(), 0);
    }

    #[test]
    fn test_clear() {
        let mut items = DynamicItems::new();
        items.calculator_item = Some(CalculatorItem {
            id: "calc".to_string(),
            expression: "2+2".to_string(),
            display_result: "4".to_string(),
            clipboard_result: Some("4".to_string()),
            is_error: false,
        });
        assert!(items.has_calculator());

        items.clear();
        assert!(!items.has_calculator());
    }

    #[test]
    fn test_process_empty_query() {
        let mut items = DynamicItems::new();
        items.process_query("", true, true, true);
        assert_eq!(items.count(), 0);
    }

    #[test]
    fn test_calculator_detection() {
        let mut items = DynamicItems::new();
        // Enable calculator, disable AI and search
        items.process_query("2+2", true, false, false);
        assert!(items.has_calculator());
        assert!(!items.has_ai());
    }
}
