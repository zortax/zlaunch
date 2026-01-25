//! Section management for the item list delegate.
//!
//! Handles organizing items into sections and converting between
//! global indices and section-based IndexPaths.

use crate::config::ConfigModule;
use crate::items::ListItem;
use gpui_component::IndexPath;

/// Section types for organizing items in the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionType {
    /// Calculator result (always first if present).
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
}

impl SectionManager {
    /// Create a new section manager.
    pub fn new(combined_modules: Vec<ConfigModule>) -> Self {
        Self {
            section_info: SectionInfo::default(),
            combined_modules,
            has_calculator: false,
            has_ai: false,
            search_count: 0,
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
        self.section_info = SectionInfo::compute(items, filtered_indices);
        self.section_info.search_count = search_count;
        self.has_calculator = has_calculator;
        self.has_ai = has_ai;
        self.search_count = search_count;
    }

    /// Get the ordered list of section types based on combined_modules.
    /// SearchAndAi is positioned at the first occurrence of either Search or Ai.
    pub fn ordered_section_types(&self) -> Vec<SectionType> {
        let mut sections = Vec::new();
        let mut seen_commands = false;
        let mut seen_search_and_ai = false;
        let has_search_and_ai = self.has_ai || self.search_count > 0;

        for module in &self.combined_modules {
            match module {
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

    /// Get the total number of sections (including calculator if present).
    pub fn sections_count(&self) -> usize {
        let mut count = 0;
        if self.has_calculator {
            count += 1;
        }
        count += self.ordered_section_types().len();
        count
    }

    /// Determine what type of section is at the given section index.
    pub fn section_type_at(&self, section: usize) -> SectionType {
        let mut current_section = 0;

        // Calculator always first
        if self.has_calculator {
            if section == current_section {
                return SectionType::Calculator;
            }
            current_section += 1;
        }

        // Sections in combined_modules order
        for section_type in self.ordered_section_types() {
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
            SectionType::Calculator => {
                if self.has_calculator {
                    1
                } else {
                    0
                }
            }
            SectionType::Windows => self.section_info.window_count,
            SectionType::Commands => self.section_info.command_count,
            SectionType::Applications => self.section_info.app_count,
            SectionType::SearchAndAi => {
                let ai_count = if self.has_ai { 1 } else { 0 };
                ai_count + self.search_count
            }
        }
    }

    /// Get the starting global index for a given section type.
    pub fn section_start_index(&self, section_type: SectionType) -> usize {
        let calc_offset = if self.has_calculator { 1 } else { 0 };

        if section_type == SectionType::Calculator {
            return 0;
        }

        let mut offset = calc_offset;
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
        let calc_offset = if self.has_calculator { 1 } else { 0 };
        let mut current_section = 0;

        // Calculator section
        if self.has_calculator {
            if global_idx < calc_offset {
                return Some(IndexPath::new(global_idx).section(current_section));
            }
            current_section += 1;
        }

        // Iterate through all sections in combined_modules order
        let mut current_start = calc_offset;

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

    #[test]
    fn test_section_type_title() {
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
}
