use crate::assets::PhosphorIcon;
use crate::process;
use crate::search::SearchProvider;

use super::traits::{Categorizable, DisplayItem, Executable, IconProvider};

/// A search item representing a web search query for a specific provider.
#[derive(Clone, Debug)]
pub struct SearchItem {
    /// Unique identifier for this item
    pub id: String,
    /// Display name (e.g., "Search on Google")
    pub name: String,
    /// The search provider
    pub provider: SearchProvider,
    /// The search query
    pub query: String,
    /// The generated search URL
    pub url: String,
}

impl SearchItem {
    /// Create a new search item for a provider and query.
    pub fn new(provider: SearchProvider, query: String) -> Self {
        let url = provider.build_url(&query);
        let id = format!("search-{}-{}", provider.name.to_lowercase(), query);
        let name = format!("Search on {}", provider.name);
        Self {
            id,
            name,
            provider,
            query,
            url,
        }
    }

    /// Get the icon for this search item.
    pub fn icon(&self) -> PhosphorIcon {
        self.provider.icon
    }
}

impl DisplayItem for SearchItem {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        None
    }

    fn action_label(&self) -> &'static str {
        "Open"
    }
}

impl IconProvider for SearchItem {
    // Uses Phosphor icons via icon() method
}

impl Executable for SearchItem {
    fn execute(&self) -> anyhow::Result<()> {
        // Open URL in browser, disowned from daemon
        process::open_url(&self.url)?;
        Ok(())
    }
}

impl Categorizable for SearchItem {
    fn section_name(&self) -> &'static str {
        "Search"
    }

    fn sort_priority(&self) -> u8 {
        1
    }
}

impl From<SearchItem> for super::ListItem {
    fn from(item: SearchItem) -> Self {
        Self::Search(item)
    }
}
