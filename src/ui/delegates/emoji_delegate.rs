use crate::emoji::EmojiItem;
use crate::ui::theme::theme;
use crate::ui::delegates::BaseDelegate;
use crate::ui::views::{render_emoji_row};
use gpui::{div, prelude::*, App, Context, SharedString, Task, Window};
use gpui_component::list::{ListDelegate, ListItem as GpuiListItem, ListState};
use gpui_component::IndexPath;

/// Delegate for the emoji picker grid.
///
/// This is a simplified delegate that composes with BaseDelegate<EmojiItem>
/// and adds grid-specific navigation logic.
pub struct EmojiGridDelegate {
    /// Base delegate handling common behavior
    base: BaseDelegate<EmojiItem>,
    /// Number of columns in the grid
    columns: usize,
}

impl EmojiGridDelegate {
    /// Create a new emoji grid delegate
    pub fn new(items: Vec<EmojiItem>, columns: usize) -> Self {
        Self {
            base: BaseDelegate::new(items),
            columns,
        }
    }

    /// Set the confirm callback
    pub fn set_on_confirm(&mut self, callback: impl Fn(&EmojiItem) + Send + Sync + 'static) {
        self.base.set_on_confirm(callback);
    }

    /// Set the cancel callback
    pub fn set_on_cancel(&mut self, callback: impl Fn() + Send + Sync + 'static) {
        self.base.set_on_cancel(callback);
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> Option<usize> {
        self.base.selected_index()
    }

    /// Get the selected row (for scrolling in grid layout)
    pub fn selected_row(&self) -> Option<usize> {
        self.selected_index().map(|idx| idx / self.columns)
    }

    /// Get the total count of filtered items
    pub fn filtered_count(&self) -> usize {
        self.base.filtered_count()
    }

    /// Get the current query
    pub fn query(&self) -> &str {
        self.base.query()
    }

    /// Clear the query
    pub fn clear_query(&mut self) {
        self.base.clear_query();
    }

    /// Set the query and filter
    pub fn set_query(&mut self, query: String) {
        self.base.set_query(query);
        self.filter_items();
    }

    /// Filter items based on the current query
    fn filter_items(&mut self) {
        let query = self.base.query();
        if query.is_empty() {
            self.base.reset_filter();
        } else {
            let items = self.base.items();
            let query_lower = query.to_lowercase();
            let filtered_indices: Vec<usize> = items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    item.emoji.contains(query)
                        || item.name.to_lowercase().contains(&query_lower)
                })
                .map(|(idx, _)| idx)
                .collect();
            self.base.apply_filtered_indices(filtered_indices);
        }
    }

    /// Get an item at a filtered index
    pub fn get_item_at(&self, index: usize) -> Option<&EmojiItem> {
        self.base.get_filtered_item(index)
    }

    /// Execute confirm callback
    pub fn do_confirm(&self) {
        self.base.do_confirm();
    }

    /// Execute cancel callback
    pub fn do_cancel(&self) {
        self.base.do_cancel();
    }

    /// Move selection right (for grid navigation)
    pub fn select_right(&mut self) {
        // In a grid, "right" is the same as "next"
        self.base.select_down();
    }

    /// Move selection left (for grid navigation)
    pub fn select_left(&mut self) {
        // In a grid, "left" is the same as "previous"
        self.base.select_up();
    }

    /// Move selection down (by one row)
    pub fn select_down(&mut self) {
        let count = self.filtered_count();
        if count == 0 {
            return;
        }

        if let Some(current) = self.selected_index() {
            let next = current + self.columns;
            if next < count {
                self.base.set_selected(next);
            } else {
                // Wrap to first item in same column
                self.base.set_selected(current % self.columns);
            }
        }
    }

    /// Move selection up (by one row)
    pub fn select_up(&mut self) {
        let count = self.filtered_count();
        if count == 0 {
            return;
        }

        if let Some(current) = self.selected_index() {
            if current >= self.columns {
                self.base.set_selected(current - self.columns);
            } else {
                // Wrap to last row in same column
                let col = current % self.columns;
                let last_row = (count - 1) / self.columns;
                let target = last_row * self.columns + col;
                if target < count {
                    self.base.set_selected(target);
                } else {
                    // If last row doesn't have this column, go to previous row
                    let prev_row = last_row - 1;
                    self.base.set_selected(prev_row * self.columns + col);
                }
            }
        }
    }

    /// Get the number of rows needed for the grid.
    fn row_count(&self) -> usize {
        let count = self.filtered_count();
        if count == 0 {
            0
        } else {
            count.div_ceil(self.columns)
        }
    }

    /// Get emojis for a specific row.
    fn emojis_for_row(&self, row: usize) -> Vec<EmojiItem> {
        let start = row * self.columns;
        let end = (start + self.columns).min(self.filtered_count());
        (start..end)
            .filter_map(|i| self.base.get_filtered_item(i).cloned())
            .collect()
    }
}

/// Implement ListDelegate trait for GPUI integration.
impl ListDelegate for EmojiGridDelegate {
    type Item = GpuiListItem;

    fn sections_count(&self, _cx: &App) -> usize {
        1
    }

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.row_count()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut Context<'_, ListState<Self>>,
    ) -> Option<Self::Item> {
        let row = ix.row;
        let emojis = self.emojis_for_row(row);
        let start_index = row * self.columns;

        let row_element = render_emoji_row(&emojis, start_index, self.base.selected_index(), self.columns);

        Some(
            GpuiListItem::new(("emoji-row", row))
                .py_0()
                .px_0()
                .child(row_element),
        )
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        // Convert row to first item in that row
        self.base.set_selected(ix.map(|i| i.row * self.columns).unwrap_or(0));
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
                    .child(SharedString::from("No emojis found")),
            )
    }
}
