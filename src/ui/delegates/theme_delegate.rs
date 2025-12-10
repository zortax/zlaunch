use crate::items::ThemeItem;
use crate::ui::delegates::BaseDelegate;
use crate::ui::theme::theme;
use crate::ui::views::render_theme_item;
use gpui::{div, prelude::*, App, Context, SharedString, Task, Window};
use gpui_component::list::{ListDelegate, ListItem as GpuiListItem, ListState};
use gpui_component::IndexPath;

/// Delegate for the theme picker list.
///
/// This delegate manages the theme list and composes with BaseDelegate<ThemeItem>.
pub struct ThemeListDelegate {
    /// Base delegate handling common behavior
    base: BaseDelegate<ThemeItem>,
}

impl ThemeListDelegate {
    /// Create a new theme list delegate
    pub fn new(items: Vec<ThemeItem>) -> Self {
        Self {
            base: BaseDelegate::new(items),
        }
    }

    /// Set the confirm callback (apply theme)
    pub fn set_on_confirm(&mut self, callback: impl Fn(&ThemeItem) + Send + Sync + 'static) {
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

    /// Set the selected index
    pub fn set_selected(&mut self, index: usize) {
        self.base.set_selected(index);
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
                    // Search in theme name
                    item.name.to_lowercase().contains(&query_lower)
                        || item.description.to_lowercase().contains(&query_lower)
                })
                .map(|(idx, _)| idx)
                .collect();
            self.base.apply_filtered_indices(filtered_indices);
        }
    }

    /// Get an item at a filtered index
    pub fn get_item_at(&self, index: usize) -> Option<&ThemeItem> {
        self.base.get_filtered_item(index)
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&ThemeItem> {
        self.base.selected_item()
    }

    /// Execute confirm callback
    pub fn do_confirm(&self) {
        self.base.do_confirm();
    }

    /// Execute cancel callback
    pub fn do_cancel(&self) {
        self.base.do_cancel();
    }

    /// Move selection down
    pub fn select_down(&mut self) {
        self.base.select_down();
    }

    /// Move selection up
    pub fn select_up(&mut self) {
        self.base.select_up();
    }

    /// Get all items
    pub fn items(&self) -> &[ThemeItem] {
        self.base.items()
    }
}

/// Implement ListDelegate trait for GPUI integration.
impl ListDelegate for ThemeListDelegate {
    type Item = GpuiListItem;

    fn sections_count(&self, _cx: &App) -> usize {
        1
    }

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.filtered_count()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut Context<'_, ListState<Self>>,
    ) -> Option<Self::Item> {
        let item = self.base.get_filtered_item(ix.row)?;
        let is_selected = self.base.selected_index() == Some(ix.row);

        let element = render_theme_item(item, is_selected, ix.row);

        // Reset ListItem default padding - we handle all styling ourselves
        Some(
            GpuiListItem::new(("theme-item", ix.row))
                .py_0()
                .px_0()
                .child(element),
        )
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        self.base.set_selected(ix.map(|i| i.row).unwrap_or(0));
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
                    .child(SharedString::from("No themes found")),
            )
    }
}
