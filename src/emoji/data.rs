use crate::emoji::EmojiItem;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use lazy_static::lazy_static;

lazy_static! {
    /// All emojis loaded from the emojis crate.
    static ref ALL_EMOJIS: Vec<EmojiItem> = load_all_emojis();
}

/// Load all emojis from the emojis crate.
fn load_all_emojis() -> Vec<EmojiItem> {
    emojis::iter()
        .map(|emoji| EmojiItem::new(emoji.as_str(), emoji.name()))
        .collect()
}

/// Get all emojis.
pub fn all_emojis() -> &'static [EmojiItem] {
    &ALL_EMOJIS
}

/// Search emojis by name using fuzzy matching.
/// Returns indices into the all_emojis() slice, sorted by match score.
pub fn search_emojis(query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..ALL_EMOJIS.len()).collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(usize, i64)> = ALL_EMOJIS
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            matcher
                .fuzzy_match(&item.name, query)
                .map(|score| (idx, score))
        })
        .collect();

    // Sort by score descending
    scored.sort_by_key(|a| std::cmp::Reverse(a.1));
    scored.into_iter().map(|(idx, _)| idx).collect()
}
