//! Clipboard history data storage and search.

use super::item::{ClipboardContent, ClipboardItem};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::collections::VecDeque;
use std::sync::RwLock;

/// Global clipboard history storage.
static CLIPBOARD_HISTORY: RwLock<Option<VecDeque<ClipboardItem>>> = RwLock::new(None);

/// Initialize the clipboard history storage.
pub fn init() {
    let mut history = CLIPBOARD_HISTORY.write().unwrap();
    if history.is_none() {
        *history = Some(VecDeque::new());
    }
}

/// Add a new item to clipboard history.
/// If the item is identical to the most recent one, it won't be added.
pub fn add_item(content: ClipboardContent) {
    let mut history = CLIPBOARD_HISTORY.write().unwrap();
    let history = history.as_mut().expect("Clipboard history not initialized");

    // Don't add duplicate consecutive items
    if let Some(last) = history.front()
        && is_same_content(&last.content, &content)
    {
        return;
    }

    let item = ClipboardItem::new(content);
    history.push_front(item);
}

/// Check if two clipboard contents are the same.
fn is_same_content(a: &ClipboardContent, b: &ClipboardContent) -> bool {
    match (a, b) {
        (ClipboardContent::Text(a), ClipboardContent::Text(b)) => a == b,
        (
            ClipboardContent::Image {
                width: w1,
                height: h1,
                rgba_bytes: b1,
            },
            ClipboardContent::Image {
                width: w2,
                height: h2,
                rgba_bytes: b2,
            },
        ) => w1 == w2 && h1 == h2 && b1 == b2,
        (ClipboardContent::FilePaths(a), ClipboardContent::FilePaths(b)) => a == b,
        (
            ClipboardContent::RichText {
                plain: p1,
                html: h1,
            },
            ClipboardContent::RichText {
                plain: p2,
                html: h2,
            },
        ) => p1 == p2 && h1 == h2,
        _ => false,
    }
}

/// Get all clipboard items, optionally filtered by a search query.
pub fn search_items(query: &str) -> Vec<ClipboardItem> {
    let history = CLIPBOARD_HISTORY.read().unwrap();
    let history = history.as_ref().expect("Clipboard history not initialized");

    if query.is_empty() {
        return history.iter().cloned().collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(ClipboardItem, i64)> = history
        .iter()
        .filter_map(|item| {
            let search_text = match &item.content {
                ClipboardContent::Text(text) => text.clone(),
                ClipboardContent::Image { .. } => "image".to_string(),
                ClipboardContent::FilePaths(paths) => paths
                    .iter()
                    .filter_map(|p| p.to_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                ClipboardContent::RichText { plain, .. } => plain.clone(),
            };

            matcher
                .fuzzy_match(&search_text, query)
                .map(|score| (item.clone(), score))
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().map(|(item, _)| item).collect()
}

/// Get the total number of items in history.
pub fn item_count() -> usize {
    let history = CLIPBOARD_HISTORY.read().unwrap();
    history.as_ref().map(|h| h.len()).unwrap_or(0)
}

/// Clear all clipboard history.
#[allow(dead_code)]
pub fn clear_history() {
    let mut history = CLIPBOARD_HISTORY.write().unwrap();
    if let Some(h) = history.as_mut() {
        h.clear();
    }
}
