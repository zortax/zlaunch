//! Search trigger detection and query parsing.
//!
//! This module provides functionality to detect if user input contains a search trigger
//! (e.g., "!g rust async") and parse out the provider and query.

use super::providers::{SearchProvider, get_providers};

/// The result of parsing a search query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchDetection {
    /// A specific provider was triggered (e.g., "!g rust")
    Triggered {
        provider: SearchProvider,
        query: String,
    },
    /// No specific provider, but we should show all as fallback
    Fallback { query: String },
    /// Not a search query
    None,
}

/// Detect if the input contains a search trigger and parse it.
///
/// Returns:
/// - `SearchDetection::Triggered` if input starts with a known trigger (e.g., "!g rust")
/// - `SearchDetection::Fallback` if input should show all providers as fallback
/// - `SearchDetection::None` if this is not a search query
pub fn detect_search(input: &str) -> SearchDetection {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return SearchDetection::None;
    }

    // Check if input starts with a trigger
    for provider in get_providers() {
        if let Some(stripped) = trimmed.strip_prefix(provider.trigger) {
            // Extract the query after the trigger
            let query = stripped.trim();

            if query.is_empty() {
                // Just the trigger, no query yet - don't show anything
                return SearchDetection::None;
            }

            return SearchDetection::Triggered {
                provider,
                query: query.to_string(),
            };
        }
    }

    // No trigger found - this could be a fallback candidate
    // The delegate will decide if it should show based on whether other results exist
    SearchDetection::Fallback {
        query: trimmed.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_triggered_google() {
        let result = detect_search("!g rust async");
        match result {
            SearchDetection::Triggered { provider, query } => {
                assert_eq!(provider.name, "Google");
                assert_eq!(query, "rust async");
            }
            _ => panic!("Expected Triggered"),
        }
    }

    #[test]
    fn test_detect_triggered_wikipedia() {
        let result = detect_search("!wiki   quantum physics  ");
        match result {
            SearchDetection::Triggered { provider, query } => {
                assert_eq!(provider.name, "Wikipedia");
                assert_eq!(query, "quantum physics");
            }
            _ => panic!("Expected Triggered"),
        }
    }

    #[test]
    fn test_detect_trigger_only() {
        let result = detect_search("!g");
        assert_eq!(result, SearchDetection::None);
    }

    #[test]
    fn test_detect_trigger_with_spaces() {
        let result = detect_search("!g   ");
        assert_eq!(result, SearchDetection::None);
    }

    #[test]
    fn test_detect_fallback() {
        let result = detect_search("some random query");
        match result {
            SearchDetection::Fallback { query } => {
                assert_eq!(query, "some random query");
            }
            _ => panic!("Expected Fallback"),
        }
    }

    #[test]
    fn test_detect_empty() {
        let result = detect_search("");
        assert_eq!(result, SearchDetection::None);
    }
}
