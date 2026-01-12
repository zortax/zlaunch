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

fn provider_icon(provider_name: &str, icon_name: Option<&String>) -> PhosphorIcon {
    if let Some(icon_name) = icon_name {
        let normalized = icon_name.trim().to_ascii_lowercase();

        if !normalized.is_empty() {
            if let Some(icon) = PhosphorIcon::from_name(&normalized) {
                return icon;
            }

            warn!(
                "Unknown icon '{}' for search provider '{}', using MagnifyingGlass",
                icon_name, provider_name
            );
        }
    }

    PhosphorIcon::MagnifyingGlass
}

/// Get all available search providers
pub fn get_providers() -> Vec<SearchProvider> {
    let mut providers = vec![];

    if let Some(custom) = config().search_providers {
        for provider in custom {
            let icon = provider_icon(&provider.name, Some(&provider.icon));

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
