//! Enhanced delegate for the main item list.
//!
//! Composes BaseDelegate with dynamic items (calculator, AI, search)
//! and section management.

use crate::ai::LLMClient;
use crate::config::{ConfigModule, config};
use crate::items::{ActionItem, ListItem, SubmenuItem};
use crate::ui::delegates::BaseDelegate;
use crate::ui::theme::theme;
use crate::ui::views::render_item;
use gpui::{App, Context, SharedString, Task, Window, div, prelude::*};
use gpui_component::IndexPath;
use gpui_component::list::{ListDelegate, ListItem as GpuiListItem, ListState};
use std::sync::Arc;

use super::dynamic_items::DynamicItems;
use super::item_filter::ItemFilter;
use super::section_manager::{SectionManager, SectionType};

/// Type alias for confirm callback.
type ConfirmCallback = Arc<dyn Fn(&ListItem) + Send + Sync>;

/// Enhanced delegate for the main item list.
///
/// This delegate composes with BaseDelegate<ListItem> and adds:
/// - Dynamic calculator results
/// - AI query detection
/// - Web search suggestions
/// - Section management
pub struct ItemListDelegate {
    /// Base delegate handling common behavior.
    base: BaseDelegate<ListItem>,
    /// Fuzzy filter for items.
    filter: ItemFilter,
    /// Dynamic items (calculator, AI, search).
    dynamic: DynamicItems,
    /// Section manager for organizing items.
    sections: SectionManager,
    /// Confirm callback (stored here to handle dynamic items).
    on_confirm: Option<ConfirmCallback>,
    /// Modules enabled in combined view (for filtering).
    combined_modules: Vec<ConfigModule>,
}

impl ItemListDelegate {
    /// Create a new item list delegate with specified combined modules.
    pub fn new(mut items: Vec<ListItem>, combined_modules: Vec<ConfigModule>) -> Self {
        // Filter items based on combined_modules
        items.retain(|item| match item {
            ListItem::Application(_) => combined_modules.contains(&ConfigModule::Applications),
            ListItem::Window(_) => combined_modules.contains(&ConfigModule::Windows),
            _ => true, // Keep other items for now
        });

        // Add built-in submenu items (only if module is in combined_modules)
        if combined_modules.contains(&ConfigModule::Emojis) {
            items.push(ListItem::Submenu(
                SubmenuItem::grid("submenu-emojis", "Emojis", 8)
                    .with_description("Search and copy emojis")
                    .with_icon("smiley"),
            ));
        }
        if combined_modules.contains(&ConfigModule::Clipboard) {
            items.push(ListItem::Submenu(
                SubmenuItem::list("submenu-clipboard", "Clipboard History")
                    .with_description("View and paste clipboard history")
                    .with_icon("clipboard"),
            ));
        }
        if combined_modules.contains(&ConfigModule::Themes) {
            items.push(ListItem::Submenu(
                SubmenuItem::list("submenu-themes", "Themes")
                    .with_description("Browse and apply themes")
                    .with_icon("palette"),
            ));
        }

        // Add built-in action items (shutdown, reboot, etc.)
        if combined_modules.contains(&ConfigModule::Actions) {
            for action in ActionItem::builtins() {
                items.push(ListItem::Action(action));
            }
        }

        // Sort items by their position in combined_modules
        tracing::debug!(?combined_modules, "Sorting items by combined_modules order");
        items.sort_by(|a, b| {
            let a_module = a.config_module();
            let b_module = b.config_module();

            let a_pos = combined_modules
                .iter()
                .position(|m| m == &a_module)
                .unwrap_or(usize::MAX);
            let b_pos = combined_modules
                .iter()
                .position(|m| m == &b_module)
                .unwrap_or(usize::MAX);

            a_pos
                .cmp(&b_pos)
                .then_with(|| a.sort_priority().cmp(&b.sort_priority()))
        });

        // Debug: show first few items after sorting
        for (i, item) in items.iter().take(5).enumerate() {
            tracing::debug!(i, name = item.name(), module = ?item.config_module(), "Sorted item");
        }

        // Get fuzzy match config from application config
        let fuzzy_config = config().fuzzy_match.clone();

        let mut sections =
            SectionManager::new(combined_modules.clone(), fuzzy_config.show_best_match);
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
        sections.update(&items, &filtered_indices, false, false, 0);

        Self {
            base: BaseDelegate::new(items),
            filter: ItemFilter::new(fuzzy_config),
            dynamic: DynamicItems::new(),
            sections,
            on_confirm: None,
            combined_modules,
        }
    }

    /// Set the confirm callback.
    pub fn set_on_confirm(&mut self, callback: impl Fn(&ListItem) + Send + Sync + 'static) {
        self.on_confirm = Some(Arc::new(callback));
    }

    /// Set the cancel callback.
    pub fn set_on_cancel(&mut self, callback: impl Fn() + Send + Sync + 'static) {
        self.base.set_on_cancel(callback);
    }

    /// Get the currently selected index.
    pub fn selected_index(&self) -> Option<usize> {
        self.base.selected_index()
    }

    /// Set the selected index (override to handle dynamic items).
    pub fn set_selected(&mut self, index: usize) {
        if index < self.filtered_count() {
            self.base.set_selected_unchecked(index);
        }
    }

    /// Get the total count of filtered items (including dynamic items).
    pub fn filtered_count(&self) -> usize {
        self.base.filtered_count() + self.dynamic.count()
    }

    /// Get the current query.
    pub fn query(&self) -> &str {
        self.base.query()
    }

    /// Clear the query and reset all dynamic items.
    pub fn clear_query(&mut self) {
        self.dynamic.clear();
        self.base.clear_query();
        // Re-filter to reset sections
        self.filter_items();
    }

    /// Set the query and trigger filtering.
    pub fn set_query(&mut self, query: String) {
        self.base.set_query(query.clone());
        self.process_query(&query);
    }

    /// Process the query to detect special items.
    fn process_query(&mut self, query: &str) {
        let ai_enabled =
            self.combined_modules.contains(&ConfigModule::Ai) && LLMClient::is_configured();
        let calculator_enabled = self.combined_modules.contains(&ConfigModule::Calculator);
        let search_enabled = self.combined_modules.contains(&ConfigModule::Search);

        // Process dynamic items
        self.dynamic
            .process_query(query, calculator_enabled, ai_enabled, search_enabled);

        // Filter the base items
        self.filter_items();

        // Ensure selection is initialized when we have items
        if self.base.selected_index().is_none() && self.filtered_count() > 0 {
            self.base.set_selected_unchecked(0);
        }
    }

    /// Filter items based on the current query.
    fn filter_items(&mut self) {
        let query = self.base.query();
        let items = self.base.items();

        // Get filtered items with scores for best-match detection
        let filtered = self
            .filter
            .filter_with_scores(items, query, &self.combined_modules);

        // Extract indices for base delegate
        let filtered_indices: Vec<usize> = filtered.iter().map(|f| f.index).collect();
        self.base.apply_filtered_indices(filtered_indices);

        // Update sections with scores
        self.sections.update_with_scores(
            self.base.items(),
            &filtered,
            self.dynamic.has_calculator(),
            self.dynamic.has_ai(),
            self.dynamic.search_count(),
        );

        // Ensure selection is initialized
        if self.base.selected_index().is_none() && self.filtered_count() > 0 {
            self.base.set_selected_unchecked(0);
        }
    }

    /// Get an item at a global index (including dynamic items).
    pub fn get_item_at(&self, global_index: usize) -> Option<ListItem> {
        // Track offset within regular items (excluding best match)
        let mut regular_item_offset = 0;
        let mut current_start = 0;

        // Get the best match position if applicable
        let best_match_pos = self.sections.best_match_filtered_pos();

        for section_type in self.sections.ordered_section_types() {
            let section_count = self.sections.section_item_count(section_type);
            let section_end = current_start + section_count;

            if global_index >= current_start && global_index < section_end {
                let row = global_index - current_start;

                return match section_type {
                    SectionType::BestMatch => {
                        // Return the promoted best match item
                        let best_pos = best_match_pos?;
                        self.base.get_filtered_item(best_pos).cloned()
                    }
                    SectionType::Calculator => self
                        .dynamic
                        .calculator_item
                        .clone()
                        .map(ListItem::Calculator),
                    SectionType::Windows | SectionType::Commands | SectionType::Applications => {
                        // Calculate the actual index, skipping the best match if it was in this section
                        let base_idx = self.get_adjusted_base_index(
                            regular_item_offset,
                            row,
                            section_type,
                            best_match_pos,
                        );
                        self.base.get_filtered_item(base_idx).cloned()
                    }
                    SectionType::SearchAndAi => {
                        let ai_count = if self.dynamic.has_ai() { 1 } else { 0 };
                        if row == 0 && self.dynamic.has_ai() {
                            self.dynamic.ai_item.clone().map(ListItem::Ai)
                        } else {
                            let search_idx = row - ai_count;
                            self.dynamic
                                .search_items
                                .get(search_idx)
                                .cloned()
                                .map(ListItem::Search)
                        }
                    }
                };
            }

            // Track offset for regular items (excluding BestMatch and Calculator)
            if matches!(
                section_type,
                SectionType::Windows | SectionType::Commands | SectionType::Applications
            ) {
                regular_item_offset += section_count;
                // Add 1 if best match was from this section (since we subtracted it from count)
                if self.sections.best_match_original_section() == Some(section_type) {
                    regular_item_offset += 1;
                }
            }
            current_start = section_end;
        }

        None
    }

    /// Get the adjusted base index, accounting for the best match being skipped.
    fn get_adjusted_base_index(
        &self,
        offset: usize,
        row: usize,
        section_type: SectionType,
        best_match_pos: Option<usize>,
    ) -> usize {
        let base_idx = offset + row;

        // If best match is from this section and we're past where it would be, add 1
        if let Some(best_pos) = best_match_pos {
            if self.sections.best_match_original_section() == Some(section_type) {
                // The best match was removed from this section
                // We need to map our row to the correct position in filtered items
                // by skipping the best match position
                if base_idx >= best_pos {
                    return base_idx + 1;
                }
            }
        }

        base_idx
    }

    /// Execute confirm callback for the selected item.
    pub fn do_confirm(&self) {
        if let Some(idx) = self.selected_index()
            && let Some(item) = self.get_item_at(idx)
            && let Some(ref callback) = self.on_confirm
        {
            callback(&item);
        }
    }

    /// Execute cancel callback.
    pub fn do_cancel(&self) {
        self.base.do_cancel();
    }

    /// Move selection down.
    pub fn select_down(&mut self) {
        let count = self.filtered_count();
        if count == 0 {
            return;
        }

        let current = self.selected_index().unwrap_or(0);
        let next = if current + 1 >= count { 0 } else { current + 1 };
        self.set_selected(next);
    }

    /// Move selection up.
    pub fn select_up(&mut self) {
        let count = self.filtered_count();
        if count == 0 {
            return;
        }

        let current = self.selected_index().unwrap_or(0);
        let prev = if current == 0 { count - 1 } else { current - 1 };
        self.set_selected(prev);
    }

    /// Get all items for external access.
    pub fn items(&self) -> Arc<Vec<ListItem>> {
        Arc::new(self.base.items().to_vec())
    }

    /// Convert global index to section+row IndexPath.
    pub fn global_to_index_path(&self, global_idx: usize) -> Option<IndexPath> {
        self.sections.global_to_index_path(global_idx)
    }
}

/// Implement ListDelegate trait for GPUI integration.
impl ListDelegate for ItemListDelegate {
    type Item = GpuiListItem;

    fn sections_count(&self, _cx: &App) -> usize {
        self.sections.sections_count()
    }

    fn items_count(&self, section: usize, _cx: &App) -> usize {
        let section_type = self.sections.section_type_at(section);
        self.sections.section_item_count(section_type)
    }

    fn render_section_header(
        &mut self,
        section: usize,
        _window: &mut Window,
        _cx: &mut Context<'_, ListState<Self>>,
    ) -> Option<impl IntoElement> {
        let section_type = self.sections.section_type_at(section);
        let sections = self.sections.ordered_section_types();
        let section_count = sections.len();

        // Don't show headers if there's only one section
        if section_count <= 1 {
            return None;
        }

        // SearchAndAi section: show header only when there are other sections
        if section_type == SectionType::SearchAndAi {
            let has_other_sections = sections.iter().any(|s| *s != SectionType::SearchAndAi);
            if !has_other_sections {
                return None;
            }
        }

        let theme = theme();
        let title = section_type.title();

        Some(
            div()
                .w_full()
                .px(theme.item_margin_x + theme.item_padding_x)
                .pt(theme.section_header.margin_top)
                .pb(theme.section_header.margin_bottom)
                .text_xs()
                .font_weight(gpui::FontWeight::EXTRA_BOLD)
                .text_color(theme.section_header.color)
                .child(SharedString::from(title)),
        )
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut Context<'_, ListState<Self>>,
    ) -> Option<Self::Item> {
        let global_idx = self.sections.section_row_to_global(ix.section, ix.row);
        let selected = self.base.selected_index() == Some(global_idx);

        let item = self.get_item_at(global_idx)?;
        let item_content = render_item(&item, selected, global_idx);

        Some(
            GpuiListItem::new(("list-item", global_idx))
                .py_0()
                .px_0()
                .child(item_content),
        )
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        let global_idx = ix
            .map(|i| self.sections.section_row_to_global(i.section, i.row))
            .unwrap_or(0);

        self.base.set_selected_unchecked(global_idx);
    }

    fn perform_search(
        &mut self,
        query: &str,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.set_query(query.to_string());
        Task::ready(())
    }

    fn confirm(
        &mut self,
        _secondary: bool,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        self.do_confirm();
    }

    fn cancel(&mut self, _window: &mut Window, _cx: &mut Context<ListState<Self>>) {
        self.do_cancel();
    }

    fn render_empty(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<'_, ListState<Self>>,
    ) -> impl IntoElement {
        let theme = theme();
        div()
            .w_full()
            .h(theme.empty_state_height)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(theme.empty_state_color)
                    .child(SharedString::from("No items found")),
            )
    }
}
