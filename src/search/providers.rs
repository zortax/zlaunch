//! Search provider definitions.
//!
//! This module defines the available search providers (Google, DuckDuckGo, Wikipedia, YouTube)
//! with their triggers, URL templates, and icons.

use crate::assets::PhosphorIcon;
use crate::config::config;
use tracing::warn;

/// A search provider configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchProvider {
    /// The provider name (e.g., "Google", "DuckDuckGo")
    pub name: String,
    /// The trigger string (e.g., "!g", "!d", "!wiki")
    pub trigger: String,
    /// The URL template with {query} placeholder
    pub url_template: String,
    /// The Phosphor icon to use
    pub icon: PhosphorIcon,
}

impl SearchProvider {
    /// Build a search URL with the given query.
    pub fn build_url(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        self.url_template.replace("{query}", &encoded_query)
    }
}

/// Get all build in search providers.
fn builtin_providers() -> Vec<SearchProvider> {
    vec![
        SearchProvider {
            name: "Google".to_string(),
            trigger: "!g".to_string(),
            url_template: "https://www.google.com/search?q={query}".to_string(),
            icon: PhosphorIcon::MagnifyingGlass,
        },
        SearchProvider {
            name: "DuckDuckGo".to_string(),
            trigger: "!d".to_string(),
            url_template: "https://duckduckgo.com/?q={query}".to_string(),
            icon: PhosphorIcon::Globe,
        },
        SearchProvider {
            name: "Wikipedia".to_string(),
            trigger: "!wiki".to_string(),
            url_template: "https://en.wikipedia.org/wiki/Special:Search?search={query}".to_string(),
            icon: PhosphorIcon::BookOpen,
        },
        SearchProvider {
            name: "YouTube".to_string(),
            trigger: "!yt".to_string(),
            url_template: "https://www.youtube.com/results?search_query={query}".to_string(),
            icon: PhosphorIcon::YoutubeLogo,
        },
    ]
}

fn provider_icon(provider_name: &str, icon_name: Option<&String>) -> PhosphorIcon {
    if let Some(icon_name) = icon_name {
        let normalized = icon_name.trim().to_ascii_lowercase();

        if let Some(icon) = PhosphorIcon::from_name(&normalized) {
            return icon;
        }

        warn!(
            "Unknown icon '{}' for search provider '{}', using magnifying-glass",
            icon_name, provider_name
        );
    }

    PhosphorIcon::MagnifyingGlass
}

/// Get all available search providers
pub fn get_providers() -> Vec<SearchProvider> {
    let mut providers = builtin_providers();

    if let Some(custom) = config().search_providers {
        for provider in custom {
            let icon = provider_icon(&provider.name, provider.icon.as_ref());

            providers.push(SearchProvider {
                name: provider.name,
                trigger: provider.trigger,
                url_template: provider.url,
                icon,
            });
        }
    }

    providers
}

/// Find a provider by its trigger.
pub fn find_provider_by_trigger(trigger: &str) -> Option<SearchProvider> {
    get_providers().into_iter().find(|p| p.trigger == trigger)
}
