//! Fuzzy filtering for list items.
//!
//! Provides fuzzy matching functionality using SkimMatcherV2 from fuzzy-matcher.
//! Respects module ordering from combined_modules configuration.

use crate::config::ConfigModule;
use crate::items::ListItem;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Fuzzy filter for list items.
pub struct ItemFilter {
    /// The fuzzy matcher instance.
    matcher: SkimMatcherV2,
}

impl Default for ItemFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemFilter {
    /// Create a new item filter.
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Filter items by query, returning indices of matching items.
    ///
    /// When query is empty, returns all indices in original order.
    /// When query is non-empty, returns indices sorted by:
    /// 1. Module position in combined_modules (primary)
    /// 2. Fuzzy score (secondary, higher is better)
    pub fn filter_with_modules(
        &self,
        items: &[ListItem],
        query: &str,
        combined_modules: &[ConfigModule],
    ) -> Vec<usize> {
        if query.is_empty() {
            return (0..items.len()).collect();
        }

        let mut scored: Vec<(usize, i64)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                let score = self.score_item(item, query)?;
                Some((idx, score))
            })
            .collect();

        // Sort by module position, then by score within same module
        scored.sort_by(|a, b| {
            let module_a = items[a.0].config_module();
            let module_b = items[b.0].config_module();

            let pos_a = combined_modules
                .iter()
                .position(|m| m == &module_a)
                .unwrap_or(usize::MAX);
            let pos_b = combined_modules
                .iter()
                .position(|m| m == &module_b)
                .unwrap_or(usize::MAX);

            // Primary: module position, Secondary: fuzzy score (higher is better)
            pos_a.cmp(&pos_b).then_with(|| b.1.cmp(&a.1))
        });

        scored.into_iter().map(|(idx, _)| idx).collect()
    }

    /// Get the fuzzy score for an item against a query.
    ///
    /// Returns the higher of the name score or description score.
    fn score_item(&self, item: &ListItem, query: &str) -> Option<i64> {
        let score_name = self.matcher.fuzzy_match(item.name(), query);
        let score_desc = self.matcher.fuzzy_match(item.description().unwrap_or(""), query);

        score_name.or(score_desc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{mock_application, mock_application_with_desc};

    #[test]
    fn test_empty_query_returns_all() {
        let filter = ItemFilter::new();
        let items: Vec<ListItem> = vec![];
        let result = filter.filter_with_modules(&items, "", &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_new() {
        let filter = ItemFilter::new();
        // Just verify it constructs without panic
        let _ = filter.matcher.fuzzy_match("test", "t");
    }

    #[test]
    fn test_empty_query_returns_all_indices() {
        let filter = ItemFilter::new();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Chrome")),
            ListItem::Application(mock_application("Code")),
        ];
        let result = filter.filter_with_modules(&items, "", &[]);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_filter_by_name() {
        let filter = ItemFilter::new();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Chrome")),
            ListItem::Application(mock_application("Code")),
        ];
        let result = filter.filter_with_modules(&items, "fire", &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0); // Firefox
    }

    #[test]
    fn test_filter_fuzzy_match() {
        let filter = ItemFilter::new();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Files")),
            ListItem::Application(mock_application("Chrome")),
        ];
        // "ff" should match "Firefox" through fuzzy matching
        let result = filter.filter_with_modules(&items, "ff", &[]);
        assert!(!result.is_empty());
        // Firefox should be in results
        assert!(result.contains(&0));
    }

    #[test]
    fn test_filter_by_description() {
        let filter = ItemFilter::new();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application_with_desc("App1", "Web Browser")),
            ListItem::Application(mock_application_with_desc("App2", "Text Editor")),
            ListItem::Application(mock_application_with_desc("App3", "File Manager")),
        ];
        let result = filter.filter_with_modules(&items, "browser", &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0); // App1 with "Web Browser" description
    }

    #[test]
    fn test_filter_no_matches() {
        let filter = ItemFilter::new();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Chrome")),
        ];
        let result = filter.filter_with_modules(&items, "zzzznotfound", &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_case_insensitive() {
        let filter = ItemFilter::new();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Chrome")),
        ];
        // Test with lowercase query matching uppercase in name
        let result = filter.filter_with_modules(&items, "firefox", &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_filter_respects_module_order() {
        let filter = ItemFilter::new();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("App Firefox")),
            ListItem::Application(mock_application("App Chrome")),
        ];
        // With Applications module first, both apps should match "App"
        let modules = vec![ConfigModule::Applications];
        let result = filter.filter_with_modules(&items, "App", &modules);
        assert_eq!(result.len(), 2);
    }
}
