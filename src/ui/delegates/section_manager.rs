//! Section management for the item list delegate.
//!
//! Handles organizing items into sections and converting between
//! global indices and section-based IndexPaths.

use crate::config::ConfigModule;
use crate::items::ListItem;
use gpui_component::IndexPath;

use super::item_filter::FilteredItem;

/// Section types for organizing items in the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionType {
    /// Best match item promoted to top (when enabled).
    BestMatch,
    /// Calculator result (always first if present, after best match).
    Calculator,
    /// Open windows.
    Windows,
    /// Submenus and actions (emojis, clipboard, themes, actions).
    Commands,
    /// Desktop applications.
    Applications,
    /// Combined Search + AI section (positioned by first occurrence in combined_modules).
    SearchAndAi,
}

impl SectionType {
    /// Get the display title for this section.
    pub fn title(&self) -> &'static str {
        match self {
            SectionType::BestMatch => "Best Match",
            SectionType::Calculator => "Calculator",
            SectionType::Windows => "Windows",
            SectionType::Commands => "Commands",
            SectionType::Applications => "Applications",
            SectionType::SearchAndAi => "Search and AI",
        }
    }
}

/// Section information for tracking item counts by type.
#[derive(Clone, Debug, Default)]
pub struct SectionInfo {
    /// Number of search items.
    pub search_count: usize,
    /// Number of window items.
    pub window_count: usize,
    /// Number of command items (submenus + actions).
    pub command_count: usize,
    /// Number of application items.
    pub app_count: usize,
}

impl SectionInfo {
    /// Compute section info from a list of items and their filtered indices.
    pub fn compute(items: &[ListItem], filtered_indices: &[usize]) -> Self {
        let mut info = SectionInfo::default();

        for &idx in filtered_indices {
            if let Some(item) = items.get(idx) {
                if item.is_window() {
                    info.window_count += 1;
                } else if item.is_submenu() || item.is_action() {
                    info.command_count += 1;
                } else if item.is_application() {
                    info.app_count += 1;
                }
            }
        }

        info
    }
}

/// Manages section organization and index conversions for the item list.
pub struct SectionManager {
    /// Current section information.
    section_info: SectionInfo,
    /// Modules in order for combined view.
    combined_modules: Vec<ConfigModule>,
    /// Whether there's a calculator item present.
    has_calculator: bool,
    /// Whether there's an AI item present.
    has_ai: bool,
    /// Number of search items.
    search_count: usize,
    /// Whether best match feature is enabled.
    show_best_match: bool,
    /// Index of best match in filtered results (if promoted).
    /// This is the position in the filtered results array, not the original items array.
    best_match_filtered_pos: Option<usize>,
    /// The section type that the best match was promoted from.
    best_match_original_section: Option<SectionType>,
}

impl SectionManager {
    /// Create a new section manager.
    pub fn new(combined_modules: Vec<ConfigModule>, show_best_match: bool) -> Self {
        Self {
            section_info: SectionInfo::default(),
            combined_modules,
            has_calculator: false,
            has_ai: false,
            search_count: 0,
            show_best_match,
            best_match_filtered_pos: None,
            best_match_original_section: None,
        }
    }

    /// Update the section info from filtered items and dynamic item state.
    pub fn update(
        &mut self,
        items: &[ListItem],
        filtered_indices: &[usize],
        has_calculator: bool,
        has_ai: bool,
        search_count: usize,
    ) {
        // Convert to FilteredItem with score 0 for backward compatibility
        let filtered: Vec<FilteredItem> = filtered_indices
            .iter()
            .map(|&index| FilteredItem { index, score: 0 })
            .collect();
        self.update_with_scores(items, &filtered, has_calculator, has_ai, search_count);
    }

    /// Update the section info from filtered items with scores.
    pub fn update_with_scores(
        &mut self,
        items: &[ListItem],
        filtered: &[FilteredItem],
        has_calculator: bool,
        has_ai: bool,
        search_count: usize,
    ) {
        let filtered_indices: Vec<usize> = filtered.iter().map(|f| f.index).collect();
        self.section_info = SectionInfo::compute(items, &filtered_indices);
        self.section_info.search_count = search_count;
        self.has_calculator = has_calculator;
        self.has_ai = has_ai;
        self.search_count = search_count;

        // Reset best match
        self.best_match_filtered_pos = None;
        self.best_match_original_section = None;

        // Determine if we should promote a best match
        if self.show_best_match && !filtered.is_empty() {
            self.compute_best_match(items, filtered);
        }
    }

    /// Compute best match promotion if applicable.
    fn compute_best_match(&mut self, items: &[ListItem], filtered: &[FilteredItem]) {
        // Find the highest scoring item
        let best = filtered.iter().enumerate().max_by_key(|(_, f)| f.score);

        let Some((best_pos, best_item)) = best else {
            return;
        };

        // If the query was empty (all scores are 0), don't promote
        if best_item.score == 0 {
            return;
        }

        let best_original_item = match items.get(best_item.index) {
            Some(item) => item,
            None => return,
        };

        let best_module = best_original_item.config_module();

        // Find the first section that has items
        let first_section_with_items = self.ordered_section_types_internal().first().cloned();

        let Some(first_section) = first_section_with_items else {
            return;
        };

        // Determine which section the best match belongs to
        let best_section = self.section_type_for_module(&best_module);

        // Only promote if best match is NOT in the first section
        if best_section != first_section {
            self.best_match_filtered_pos = Some(best_pos);
            self.best_match_original_section = Some(best_section);
        }
    }

    /// Map a ConfigModule to its SectionType.
    fn section_type_for_module(&self, module: &ConfigModule) -> SectionType {
        match module {
            ConfigModule::Windows => SectionType::Windows,
            ConfigModule::Applications => SectionType::Applications,
            ConfigModule::Search | ConfigModule::Ai => SectionType::SearchAndAi,
            ConfigModule::Actions
            | ConfigModule::Emojis
            | ConfigModule::Clipboard
            | ConfigModule::Themes => SectionType::Commands,
            ConfigModule::Calculator => SectionType::Calculator,
        }
    }

    /// Check if a best match is being promoted.
    pub fn has_best_match(&self) -> bool {
        self.best_match_filtered_pos.is_some()
    }

    /// Get the filtered position of the best match (if promoted).
    pub fn best_match_filtered_pos(&self) -> Option<usize> {
        self.best_match_filtered_pos
    }

    /// Get the section the best match was promoted from.
    pub fn best_match_original_section(&self) -> Option<SectionType> {
        self.best_match_original_section
    }

    /// Internal helper to get ordered sections without BestMatch.
    fn ordered_section_types_internal(&self) -> Vec<SectionType> {
        let mut sections = Vec::new();
        let mut seen_commands = false;
        let mut seen_search_and_ai = false;
        let has_search_and_ai = self.has_ai || self.search_count > 0;

        for module in &self.combined_modules {
            match module {
                ConfigModule::Calculator if self.has_calculator => {
                    if !sections.contains(&SectionType::Calculator) {
                        sections.push(SectionType::Calculator);
                    }
                }
                ConfigModule::Windows if self.section_info.window_count > 0 => {
                    if !sections.contains(&SectionType::Windows) {
                        sections.push(SectionType::Windows);
                    }
                }
                ConfigModule::Applications if self.section_info.app_count > 0 => {
                    if !sections.contains(&SectionType::Applications) {
                        sections.push(SectionType::Applications);
                    }
                }
                // Search and AI are combined into one section, positioned at first occurrence
                ConfigModule::Search | ConfigModule::Ai
                    if has_search_and_ai && !seen_search_and_ai =>
                {
                    sections.push(SectionType::SearchAndAi);
                    seen_search_and_ai = true;
                }
                // Actions, Emojis, Clipboard, Themes all map to Commands section
                ConfigModule::Actions
                | ConfigModule::Emojis
                | ConfigModule::Clipboard
                | ConfigModule::Themes
                    if self.section_info.command_count > 0 && !seen_commands =>
                {
                    sections.push(SectionType::Commands);
                    seen_commands = true;
                }
                _ => {}
            }
        }

        sections
    }

    /// Get the ordered list of section types based on combined_modules.
    /// If a best match is promoted, BestMatch appears first.
    pub fn ordered_section_types(&self) -> Vec<SectionType> {
        let mut sections = Vec::new();

        // Add BestMatch section if we have a promoted item
        if self.has_best_match() {
            sections.push(SectionType::BestMatch);
        }

        // Add the rest of the sections
        sections.extend(self.ordered_section_types_internal());

        sections
    }

    /// Get the total number of sections (including calculator and best match if present).
    pub fn sections_count(&self) -> usize {
        let mut count = 0;
        if self.has_best_match() {
            count += 1;
        }
        count += self.ordered_section_types_internal().len();
        count
    }

    /// Determine what type of section is at the given section index.
    pub fn section_type_at(&self, section: usize) -> SectionType {
        let mut current_section = 0;

        // BestMatch always first (if present)
        if self.has_best_match() {
            if section == current_section {
                return SectionType::BestMatch;
            }
            current_section += 1;
        }

        // Sections in combined_modules order
        for section_type in self.ordered_section_types_internal() {
            if section == current_section {
                return section_type;
            }
            current_section += 1;
        }

        // Default (shouldn't reach here)
        SectionType::Applications
    }

    /// Get the number of items in a section type.
    pub fn section_item_count(&self, section_type: SectionType) -> usize {
        match section_type {
            SectionType::BestMatch => {
                if self.has_best_match() {
                    1
                } else {
                    0
                }
            }
            SectionType::Calculator => {
                if self.has_calculator {
                    1
                } else {
                    0
                }
            }
            SectionType::Windows => {
                let count = self.section_info.window_count;
                // Subtract 1 if best match was from this section
                if self.best_match_original_section == Some(SectionType::Windows) {
                    count.saturating_sub(1)
                } else {
                    count
                }
            }
            SectionType::Commands => {
                let count = self.section_info.command_count;
                if self.best_match_original_section == Some(SectionType::Commands) {
                    count.saturating_sub(1)
                } else {
                    count
                }
            }
            SectionType::Applications => {
                let count = self.section_info.app_count;
                if self.best_match_original_section == Some(SectionType::Applications) {
                    count.saturating_sub(1)
                } else {
                    count
                }
            }
            SectionType::SearchAndAi => {
                let ai_count = if self.has_ai { 1 } else { 0 };
                let count = ai_count + self.search_count;
                if self.best_match_original_section == Some(SectionType::SearchAndAi) {
                    count.saturating_sub(1)
                } else {
                    count
                }
            }
        }
    }

    /// Get the starting global index for a given section type.
    pub fn section_start_index(&self, section_type: SectionType) -> usize {
        let mut offset = 0;

        for st in self.ordered_section_types() {
            if st == section_type {
                return offset;
            }
            offset += self.section_item_count(st);
        }

        offset
    }

    /// Convert section+row to global index.
    pub fn section_row_to_global(&self, section: usize, row: usize) -> usize {
        let section_type = self.section_type_at(section);
        self.section_start_index(section_type) + row
    }

    /// Convert global index to section+row IndexPath.
    pub fn global_to_index_path(&self, global_idx: usize) -> Option<IndexPath> {
        let mut current_section = 0;
        let mut current_start = 0;

        for section_type in self.ordered_section_types() {
            let section_count = self.section_item_count(section_type);
            let section_end = current_start + section_count;

            if section_count > 0 {
                if global_idx >= current_start && global_idx < section_end {
                    return Some(
                        IndexPath::new(global_idx - current_start).section(current_section),
                    );
                }
                current_section += 1;
            }
            current_start = section_end;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{mock_application, mock_window};

    #[test]
    fn test_section_type_title() {
        assert_eq!(SectionType::BestMatch.title(), "Best Match");
        assert_eq!(SectionType::Calculator.title(), "Calculator");
        assert_eq!(SectionType::Windows.title(), "Windows");
        assert_eq!(SectionType::SearchAndAi.title(), "Search and AI");
    }

    #[test]
    fn test_section_info_compute() {
        // Empty case
        let info = SectionInfo::compute(&[], &[]);
        assert_eq!(info.window_count, 0);
        assert_eq!(info.app_count, 0);
    }

    #[test]
    fn test_best_match_disabled() {
        let manager = SectionManager::new(
            vec![ConfigModule::Windows, ConfigModule::Applications],
            false,
        );
        assert!(!manager.has_best_match());
    }

    #[test]
    fn test_best_match_promotion() {
        // Setup: Windows first, then Applications
        let mut manager = SectionManager::new(
            vec![ConfigModule::Windows, ConfigModule::Applications],
            true, // best match enabled
        );

        let items: Vec<ListItem> = vec![
            ListItem::Window(mock_window("GitHub - Firefox", "firefox")),
            ListItem::Application(mock_application("Firefox")),
        ];

        // Simulate filtered results where App has higher score
        let filtered = vec![
            FilteredItem {
                index: 0,
                score: 50,
            }, // Window, low score
            FilteredItem {
                index: 1,
                score: 150,
            }, // App, high score (best match)
        ];

        manager.update_with_scores(&items, &filtered, false, false, 0);

        // Best match should be detected (App from Applications section)
        assert!(manager.has_best_match());
        assert_eq!(manager.best_match_filtered_pos(), Some(1));
        assert_eq!(
            manager.best_match_original_section(),
            Some(SectionType::Applications)
        );

        // Section order should be: BestMatch, Windows, Applications
        let sections = manager.ordered_section_types();
        assert_eq!(sections[0], SectionType::BestMatch);
        assert!(sections.contains(&SectionType::Windows));
        assert!(sections.contains(&SectionType::Applications));
    }

    #[test]
    fn test_best_match_already_first() {
        // Setup: Windows first, then Applications
        let mut manager = SectionManager::new(
            vec![ConfigModule::Windows, ConfigModule::Applications],
            true,
        );

        let items: Vec<ListItem> = vec![
            ListItem::Window(mock_window("Firefox - Window", "firefox")),
            ListItem::Application(mock_application("Chrome")),
        ];

        // Window has highest score and is already in first section
        let filtered = vec![
            FilteredItem {
                index: 0,
                score: 150,
            }, // Window, high score
            FilteredItem {
                index: 1,
                score: 50,
            }, // App, low score
        ];

        manager.update_with_scores(&items, &filtered, false, false, 0);

        // No best match promotion needed
        assert!(!manager.has_best_match());
        assert_eq!(manager.best_match_filtered_pos(), None);
    }

    #[test]
    fn test_best_match_empty_query() {
        let mut manager = SectionManager::new(
            vec![ConfigModule::Windows, ConfigModule::Applications],
            true,
        );

        let items: Vec<ListItem> = vec![
            ListItem::Window(mock_window("Window", "window")),
            ListItem::Application(mock_application("App")),
        ];

        // Empty query: all scores are 0
        let filtered = vec![
            FilteredItem { index: 0, score: 0 },
            FilteredItem { index: 1, score: 0 },
        ];

        manager.update_with_scores(&items, &filtered, false, false, 0);

        // No promotion for empty query
        assert!(!manager.has_best_match());
    }

    #[test]
    fn test_section_item_count_with_best_match() {
        let mut manager = SectionManager::new(
            vec![ConfigModule::Windows, ConfigModule::Applications],
            true,
        );

        let items: Vec<ListItem> = vec![
            ListItem::Window(mock_window("Window 1", "window")),
            ListItem::Application(mock_application("App 1")),
            ListItem::Application(mock_application("App 2")),
        ];

        // App 1 has highest score
        let filtered = vec![
            FilteredItem {
                index: 0,
                score: 50,
            },
            FilteredItem {
                index: 1,
                score: 150,
            }, // Best match
            FilteredItem {
                index: 2,
                score: 100,
            },
        ];

        manager.update_with_scores(&items, &filtered, false, false, 0);

        assert!(manager.has_best_match());
        // BestMatch section has 1 item
        assert_eq!(manager.section_item_count(SectionType::BestMatch), 1);
        // Windows still has 1 item
        assert_eq!(manager.section_item_count(SectionType::Windows), 1);
        // Applications has 2 - 1 (promoted) = 1 item
        assert_eq!(manager.section_item_count(SectionType::Applications), 1);
    }
}
