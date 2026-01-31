//! Fuzzy filtering for list items.
//!
//! Provides enhanced fuzzy matching functionality using SkimMatcherV2 from fuzzy-matcher.
//! The scoring algorithm rewards:
//! - Exact name matches
//! - Prefix matches (name starts with query)
//! - Word prefix matches (query matches start of any word)
//! - Contiguous character matches
//!
//! And penalizes:
//! - Description-only matches (name doesn't match, only description does)
//! - Action/submenu items in combined mode (demotes system actions)

use crate::config::{ConfigModule, FuzzyMatchConfig};
use crate::items::ListItem;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// A filtered item with its index and score.
#[derive(Debug, Clone, Copy)]
pub struct FilteredItem {
    /// Index into the original items array.
    pub index: usize,
    /// Fuzzy match score (higher is better).
    pub score: i64,
}

/// Fuzzy filter for list items with enhanced scoring.
pub struct ItemFilter {
    /// The fuzzy matcher instance.
    matcher: SkimMatcherV2,
    /// Configuration for scoring adjustments.
    pub config: FuzzyMatchConfig,
}

impl Default for ItemFilter {
    fn default() -> Self {
        Self::new(FuzzyMatchConfig::default())
    }
}

impl ItemFilter {
    /// Create a new item filter with the given configuration.
    pub fn new(config: FuzzyMatchConfig) -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            config,
        }
    }

    /// Filter items by query, returning indices of matching items.
    ///
    /// This is a convenience method that wraps `filter_with_scores`
    /// and returns only the indices.
    #[cfg(test)]
    pub fn filter_indices(
        &self,
        items: &[ListItem],
        query: &str,
        combined_modules: &[ConfigModule],
    ) -> Vec<usize> {
        self.filter_with_scores(items, query, combined_modules)
            .into_iter()
            .map(|f| f.index)
            .collect()
    }

    /// Filter items by query, returning items with their scores.
    ///
    /// This is used for best-match detection where we need to know
    /// the score of each item to determine which should be promoted.
    ///
    /// When query is empty, returns all items with score 0.
    /// When query is non-empty, returns matching items sorted by:
    /// 1. Module position in combined_modules (primary)
    /// 2. Enhanced fuzzy score (secondary, higher is better)
    pub fn filter_with_scores(
        &self,
        items: &[ListItem],
        query: &str,
        combined_modules: &[ConfigModule],
    ) -> Vec<FilteredItem> {
        if query.is_empty() {
            return (0..items.len())
                .map(|index| FilteredItem { index, score: 0 })
                .collect();
        }

        let mut scored: Vec<FilteredItem> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                let score = self.score_item(item, query)?;
                Some(FilteredItem { index: idx, score })
            })
            .collect();

        // Sort by module position, then by score within same module
        scored.sort_by(|a, b| {
            let module_a = items[a.index].config_module();
            let module_b = items[b.index].config_module();

            let pos_a = combined_modules
                .iter()
                .position(|m| m == &module_a)
                .unwrap_or(usize::MAX);
            let pos_b = combined_modules
                .iter()
                .position(|m| m == &module_b)
                .unwrap_or(usize::MAX);

            // Primary: module position, Secondary: fuzzy score (higher is better)
            pos_a.cmp(&pos_b).then_with(|| b.score.cmp(&a.score))
        });

        scored
    }

    /// Get the enhanced fuzzy score for an item against a query.
    ///
    /// The scoring algorithm:
    /// 1. Try matching against the name first (preferred)
    /// 2. Fall back to description match with penalty
    /// 3. Apply bonuses for exact/prefix/contiguous matches
    /// 4. Apply item type multipliers (demote actions/submenus)
    fn score_item(&self, item: &ListItem, query: &str) -> Option<i64> {
        let name = item.name();

        // Try name match first (preferred)
        if let Some(score) = self.score_text_match(name, query, item, false) {
            return Some(score);
        }

        // Fall back to description match (with penalty)
        if let Some(desc) = item.description() {
            if let Some(score) = self.score_text_match(desc, query, item, true) {
                return Some(score);
            }
        }

        None
    }

    /// Score a text match against a query, trying multiple query normalizations.
    ///
    /// Handles cases like "counter strike" matching "Counter-Strike" by:
    /// 1. Trying the original query
    /// 2. Trying with spaces removed (e.g., "counterstrike")
    /// 3. Trying with spaces replaced by hyphens (e.g., "counter-strike")
    fn score_text_match(
        &self,
        text: &str,
        query: &str,
        item: &ListItem,
        is_description: bool,
    ) -> Option<i64> {
        let query_lower = query.to_lowercase();
        let text_lower = text.to_lowercase();

        // Try original query first
        let match_result = self.matcher.fuzzy_indices(text, query);

        // If no match and query contains spaces, try normalized versions
        let match_result = match_result.or_else(|| {
            if query.contains(' ') {
                // Try with spaces removed: "counter strike" -> "counterstrike"
                let no_spaces: String = query.chars().filter(|c| *c != ' ').collect();
                if let Some(result) = self.matcher.fuzzy_indices(text, &no_spaces) {
                    return Some(result);
                }

                // Try with spaces as hyphens: "counter strike" -> "counter-strike"
                let with_hyphens = query.replace(' ', "-");
                if let Some(result) = self.matcher.fuzzy_indices(text, &with_hyphens) {
                    return Some(result);
                }
            }
            None
        });

        let (base_score, indices) = match_result?;
        let mut score = base_score;

        // Apply bonuses only for name matches, not descriptions
        if !is_description {
            // Exact match bonus (highest priority)
            if text_lower == query_lower {
                score += self.config.exact_match_bonus;
            }
            // Prefix match bonus
            else if text_lower.starts_with(&query_lower) {
                score += self.config.prefix_match_bonus;
            }
            // Word prefix bonus (query matches start of any word)
            else if self.matches_word_start(text, query) {
                score += self.config.word_prefix_bonus;
            }
        }

        // Contiguity bonus based on how adjacent matched characters are
        score += self.calculate_contiguity_bonus(&indices);

        // Apply description penalty if this is a description match
        if is_description {
            score = (score as f64 * self.config.description_penalty) as i64;
        }

        // Apply item type multiplier (demotes actions/submenus)
        score = self.apply_item_multiplier(score, item);

        Some(score)
    }

    /// Calculate bonus based on how contiguous (adjacent) the matched characters are.
    ///
    /// Returns a value between 0 and `contiguity_bonus` config value.
    /// Fully contiguous matches get the full bonus, scattered matches get less.
    fn calculate_contiguity_bonus(&self, indices: &[usize]) -> i64 {
        if indices.len() <= 1 {
            // Single character or empty - give full bonus
            return self.config.contiguity_bonus;
        }

        let mut adjacent_count = 0;
        for window in indices.windows(2) {
            if window[1] == window[0] + 1 {
                adjacent_count += 1;
            }
        }

        let contiguity_ratio = adjacent_count as f64 / (indices.len() - 1) as f64;
        (contiguity_ratio * self.config.contiguity_bonus as f64) as i64
    }

    /// Check if the query matches the start of any word in the text.
    fn matches_word_start(&self, text: &str, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        text.split_whitespace()
            .any(|word| word.to_lowercase().starts_with(&query_lower))
    }

    /// Apply item type multiplier to demote certain item types.
    fn apply_item_multiplier(&self, score: i64, item: &ListItem) -> i64 {
        let multiplier = match item {
            ListItem::Action(_) => self.config.action_score_multiplier,
            ListItem::Submenu(_) => self.config.submenu_score_multiplier,
            _ => 1.0,
        };
        (score as f64 * multiplier) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{ActionItem, ActionKind};
    use crate::test_utils::{mock_application, mock_application_with_desc};

    #[test]
    fn test_empty_query_returns_all() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![];
        let result = filter.filter_indices(&items, "", &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_new() {
        let filter = ItemFilter::default();
        // Just verify it constructs without panic
        let _ = filter.matcher.fuzzy_match("test", "t");
    }

    #[test]
    fn test_empty_query_returns_all_indices() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Chrome")),
            ListItem::Application(mock_application("Code")),
        ];
        let result = filter.filter_indices(&items, "", &[]);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_filter_by_name() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Chrome")),
            ListItem::Application(mock_application("Code")),
        ];
        let result = filter.filter_indices(&items, "fire", &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0); // Firefox
    }

    #[test]
    fn test_filter_fuzzy_match() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Files")),
            ListItem::Application(mock_application("Chrome")),
        ];
        // "ff" should match "Firefox" through fuzzy matching
        let result = filter.filter_indices(&items, "ff", &[]);
        assert!(!result.is_empty());
        // Firefox should be in results
        assert!(result.contains(&0));
    }

    #[test]
    fn test_filter_by_description() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application_with_desc("App1", "Web Browser")),
            ListItem::Application(mock_application_with_desc("App2", "Text Editor")),
            ListItem::Application(mock_application_with_desc("App3", "File Manager")),
        ];
        let result = filter.filter_indices(&items, "browser", &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0); // App1 with "Web Browser" description
    }

    #[test]
    fn test_filter_no_matches() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Chrome")),
        ];
        let result = filter.filter_indices(&items, "zzzznotfound", &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_case_insensitive() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Chrome")),
        ];
        // Test with lowercase query matching uppercase in name
        let result = filter.filter_indices(&items, "firefox", &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_filter_respects_module_order() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("App Firefox")),
            ListItem::Application(mock_application("App Chrome")),
        ];
        // With Applications module first, both apps should match "App"
        let modules = vec![ConfigModule::Applications];
        let result = filter.filter_indices(&items, "App", &modules);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_exact_match_beats_description_match() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Eden")),
            ListItem::Action(ActionItem::builtin(ActionKind::Logout)), // "End the session"
        ];
        // "eden" exactly matches the app name, but also fuzzy-matches "End the session"
        // The app should rank first due to exact match bonus
        let result = filter.filter_indices(&items, "eden", &[]);
        assert!(!result.is_empty());
        assert_eq!(result[0], 0); // Eden app should be first
    }

    #[test]
    fn test_name_match_beats_description_match() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application_with_desc("Browser", "Web application")),
            ListItem::Application(mock_application_with_desc("Editor", "Browser for files")),
        ];
        // "browser" matches first item's name, second item's description
        // Name match should rank higher
        let result = filter.filter_indices(&items, "browser", &[]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 0); // Browser (name match) first
    }

    #[test]
    fn test_contiguous_match_beats_scattered() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")), // fire = contiguous
            ListItem::Application(mock_application("FooInRExact")), // f-i-r-e = scattered
        ];
        let result = filter.filter_indices(&items, "fire", &[]);
        assert!(!result.is_empty());
        // Firefox should rank higher due to contiguous match
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_prefix_match_bonus() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Firefox")),
            ListItem::Application(mock_application("Waterfox")),
        ];
        // "fire" is a prefix of Firefox but in the middle of Waterfox
        let result = filter.filter_indices(&items, "fire", &[]);
        assert!(!result.is_empty());
        // Firefox should rank higher due to prefix bonus
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_action_score_multiplier() {
        // Test that the action multiplier is applied
        // We create an action with a custom name to control the test
        let filter = ItemFilter::default();

        let custom_action = ActionItem::new(
            "test-action".to_string(),
            "TestAction".to_string(),
            Some("A test action".to_string()),
            None,
            ActionKind::Command("echo test".to_string()),
        );

        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("TestAction")), // Exact match
            ListItem::Action(custom_action),                       // Same exact match, but action
        ];

        // Both have exact match on "testaction", but action should score lower due to 0.8x multiplier
        let result = filter.filter_indices(&items, "testaction", &[]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 0); // App should be first due to action's 0.8x multiplier
    }

    #[test]
    fn test_contiguity_bonus_calculation() {
        let filter = ItemFilter::default();

        // Fully contiguous: all adjacent
        let indices_contiguous = vec![0, 1, 2, 3];
        let bonus_contiguous = filter.calculate_contiguity_bonus(&indices_contiguous);

        // Scattered: no adjacent pairs
        let indices_scattered = vec![0, 5, 10, 15];
        let bonus_scattered = filter.calculate_contiguity_bonus(&indices_scattered);

        // Contiguous should get more bonus
        assert!(bonus_contiguous > bonus_scattered);
        assert_eq!(bonus_contiguous, filter.config.contiguity_bonus);
        assert_eq!(bonus_scattered, 0);
    }

    #[test]
    fn test_word_prefix_bonus() {
        let filter = ItemFilter::default();

        // "code" matches start of second word in "Visual Studio Code"
        assert!(filter.matches_word_start("Visual Studio Code", "code"));
        assert!(filter.matches_word_start("Visual Studio Code", "studio"));
        assert!(filter.matches_word_start("Visual Studio Code", "visual"));

        // "ode" doesn't match start of any word
        assert!(!filter.matches_word_start("Visual Studio Code", "ode"));
    }

    #[test]
    fn test_custom_config() {
        let config = FuzzyMatchConfig {
            description_penalty: 0.1,     // Very low description weight
            action_score_multiplier: 0.5, // Very low action weight
            ..Default::default()
        };
        let filter = ItemFilter::new(config);

        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("TestApp")),
            ListItem::Action(ActionItem::builtin(ActionKind::Logout)),
        ];

        // With low action multiplier, even if "log" matches Logout better,
        // the multiplier should significantly reduce its score
        let result = filter.filter_indices(&items, "log", &[]);
        // Logout action should still match but with reduced score
        assert!(result.contains(&1));
    }

    #[test]
    fn test_space_normalized_to_hyphen() {
        // Test that "counter strike" matches "Counter-Strike: Global Offensive"
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![ListItem::Application(mock_application(
            "Counter-Strike: Global Offensive",
        ))];

        // Should match with space in query even though name has hyphen
        let result = filter.filter_indices(&items, "counter strike", &[]);
        assert_eq!(
            result.len(),
            1,
            "Should match 'counter strike' to 'Counter-Strike'"
        );
        assert_eq!(result[0], 0);

        // Should also match without space
        let result2 = filter.filter_indices(&items, "counterstrike", &[]);
        assert_eq!(result2.len(), 1);

        // And with hyphen
        let result3 = filter.filter_indices(&items, "counter-strike", &[]);
        assert_eq!(result3.len(), 1);
    }

    #[test]
    fn test_multi_word_query() {
        let filter = ItemFilter::default();
        let items: Vec<ListItem> = vec![
            ListItem::Application(mock_application("Visual Studio Code")),
            ListItem::Application(mock_application("Android Studio")),
        ];

        // "visual studio" should match "Visual Studio Code"
        let result = filter.filter_indices(&items, "visual studio", &[]);
        assert!(result.contains(&0), "Should match 'Visual Studio Code'");

        // "android studio" should match "Android Studio"
        let result2 = filter.filter_indices(&items, "android studio", &[]);
        assert!(result2.contains(&1), "Should match 'Android Studio'");
    }
}
