use crate::items::ListItem;
use crate::ui::items::render_item;
use crate::ui::theme::theme;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use gpui::{App, Context, SharedString, Task, Window, div, prelude::*};
use gpui_component::IndexPath;
use gpui_component::list::{ListDelegate, ListItem as GpuiListItem, ListState};
use std::sync::Arc;

/// Section information for the list.
#[derive(Clone, Debug, Default)]
pub struct SectionInfo {
    /// Number of windows in filtered results
    pub window_count: usize,
    /// Number of applications in filtered results
    pub app_count: usize,
}

/// A generic delegate for displaying and filtering list items.
pub struct ItemListDelegate {
    items: Arc<Vec<ListItem>>,
    filtered_indices: Vec<usize>,
    section_info: SectionInfo,
    selected_index: Option<usize>,
    query: String,
    on_confirm: Option<Arc<dyn Fn(&ListItem) + Send + Sync>>,
    on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ItemListDelegate {
    pub fn new(items: Vec<ListItem>) -> Self {
        let len = items.len();
        let filtered_indices: Vec<usize> = (0..len).collect();
        let section_info = Self::compute_section_info(&items, &filtered_indices);

        Self {
            items: Arc::new(items),
            filtered_indices,
            section_info,
            selected_index: if len > 0 { Some(0) } else { None },
            query: String::new(),
            on_confirm: None,
            on_cancel: None,
        }
    }

    /// Compute section counts from filtered indices.
    fn compute_section_info(items: &[ListItem], filtered_indices: &[usize]) -> SectionInfo {
        let mut info = SectionInfo::default();

        for &item_idx in filtered_indices {
            if let Some(item) = items.get(item_idx) {
                if item.is_window() {
                    info.window_count += 1;
                } else if item.is_application() {
                    info.app_count += 1;
                }
            }
        }

        info
    }

    /// Set the callback for when an item is confirmed (Enter pressed).
    pub fn set_on_confirm(&mut self, callback: impl Fn(&ListItem) + Send + Sync + 'static) {
        self.on_confirm = Some(Arc::new(callback));
    }

    /// Set the callback for when the list is cancelled (Escape pressed).
    pub fn set_on_cancel(&mut self, callback: impl Fn() + Send + Sync + 'static) {
        self.on_cancel = Some(Arc::new(callback));
    }

    /// Returns the items Arc for use in background filtering.
    pub fn items(&self) -> Arc<Vec<ListItem>> {
        Arc::clone(&self.items)
    }

    /// Filter items on a background thread - returns filtered indices.
    /// Results are sorted by type (windows first) then by score.
    pub fn filter_items_sync(items: &[ListItem], query: &str) -> Vec<usize> {
        if query.is_empty() {
            // Sort by type priority (windows first, then applications)
            let mut indices: Vec<usize> = (0..items.len()).collect();
            indices.sort_by_key(|&idx| items[idx].sort_priority());
            indices
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored: Vec<(usize, i64)> = items
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    matcher
                        .fuzzy_match(item.name(), query)
                        .map(|score| (idx, score))
                })
                .collect();

            // Sort by type priority first, then by score within each type
            scored.sort_by(|a, b| {
                let priority_a = items[a.0].sort_priority();
                let priority_b = items[b.0].sort_priority();
                priority_a.cmp(&priority_b).then_with(|| b.1.cmp(&a.1))
            });
            scored.into_iter().map(|(idx, _)| idx).collect()
        }
    }

    /// Apply pre-computed filter results.
    pub fn apply_filter_results(&mut self, query: String, indices: Vec<usize>) {
        // Only apply if query still matches (user might have typed more)
        if self.query == query {
            self.section_info = Self::compute_section_info(&self.items, &indices);
            self.filtered_indices = indices;
            self.selected_index = if self.filtered_indices.is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    fn filter_items(&mut self) {
        self.filtered_indices = Self::filter_items_sync(&self.items, &self.query);
        self.section_info = Self::compute_section_info(&self.items, &self.filtered_indices);
        self.selected_index = if self.filtered_indices.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    fn get_item_at(&self, row: usize) -> Option<&ListItem> {
        self.filtered_indices
            .get(row)
            .and_then(|&idx| self.items.get(idx))
    }

    /// Get item at a specific section and row index.
    fn get_item_at_section(&self, section: usize, row: usize) -> Option<&ListItem> {
        // Section 0 = windows (if any), Section 1 = applications (if windows exist)
        // If no windows, Section 0 = applications
        let has_windows = self.section_info.window_count > 0;

        let global_row = if has_windows {
            if section == 0 {
                row // Windows section
            } else {
                self.section_info.window_count + row // Applications section
            }
        } else {
            row // Only applications
        };

        self.filtered_indices
            .get(global_row)
            .and_then(|&idx| self.items.get(idx))
    }

    /// Convert section + row to global selected index.
    fn section_row_to_global(&self, section: usize, row: usize) -> usize {
        let has_windows = self.section_info.window_count > 0;
        if has_windows && section == 1 {
            self.section_info.window_count + row
        } else {
            row
        }
    }

    /// Convert global index to section + row.
    pub fn global_to_section_row(&self, global: usize) -> (usize, usize) {
        let has_windows = self.section_info.window_count > 0;
        if has_windows && global >= self.section_info.window_count {
            (1, global - self.section_info.window_count)
        } else {
            (0, global)
        }
    }

    pub fn clear_query(&mut self) {
        self.query.clear();
        self.filter_items();
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.filter_items();
    }

    /// Set query without filtering (for async filtering).
    pub fn set_query_only(&mut self, query: String) {
        self.query = query;
    }

    /// Get current query.
    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn set_selected(&mut self, index: usize) {
        self.selected_index = Some(index);
    }

    pub fn do_confirm(&self) {
        if let Some(idx) = self.selected_index
            && let Some(item) = self.get_item_at(idx)
            && let Some(ref on_confirm) = self.on_confirm
        {
            on_confirm(item);
        }
    }

    pub fn do_cancel(&self) {
        if let Some(ref on_cancel) = self.on_cancel {
            on_cancel();
        }
    }
}

impl ListDelegate for ItemListDelegate {
    type Item = GpuiListItem;

    fn sections_count(&self, _cx: &App) -> usize {
        let has_windows = self.section_info.window_count > 0;
        let has_apps = self.section_info.app_count > 0;

        match (has_windows, has_apps) {
            (true, true) => 2,   // Windows + Applications
            (true, false) => 1,  // Only Windows
            (false, true) => 1,  // Only Applications
            (false, false) => 0, // Empty
        }
    }

    fn items_count(&self, section: usize, _cx: &App) -> usize {
        let has_windows = self.section_info.window_count > 0;

        if has_windows {
            if section == 0 {
                self.section_info.window_count
            } else {
                self.section_info.app_count
            }
        } else {
            self.section_info.app_count
        }
    }

    fn render_section_header(
        &self,
        section: usize,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<impl IntoElement> {
        let has_windows = self.section_info.window_count > 0;
        let has_apps = self.section_info.app_count > 0;

        // Only show headers if we have both types
        if !has_windows || !has_apps {
            return None;
        }

        let t = theme();
        let title = if section == 0 {
            "Windows"
        } else {
            "Applications"
        };

        Some(
            div()
                .w_full()
                .px(t.item_margin_x + t.item_padding_x)
                .pt(t.section_header_margin_top)
                .pb(t.section_header_margin_bottom)
                .text_xs()
                .font_weight(gpui::FontWeight::EXTRA_BOLD)
                .text_color(t.section_header_color)
                .child(SharedString::from(title)),
        )
    }

    fn render_item(
        &self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<Self::Item> {
        let item = self.get_item_at_section(ix.section, ix.row)?;
        let global_idx = self.section_row_to_global(ix.section, ix.row);
        let selected = self.selected_index == Some(global_idx);

        let item_content = render_item(item, selected, global_idx);

        // Reset ListItem default padding - we handle all styling ourselves
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
        self.selected_index = ix.map(|i| self.section_row_to_global(i.section, i.row));
    }

    fn perform_search(
        &mut self,
        query: &str,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.query = query.to_string();
        self.filter_items();
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

    fn render_empty(&self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let t = theme();
        div()
            .w_full()
            .h(t.empty_state_height)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(t.empty_state_color)
                    .child(SharedString::from("No items found")),
            )
    }
}
