use gpui::{Context, ScrollStrategy, Window};
use gpui_component::list::{ListDelegate, ListState};
use gpui_component::IndexPath;

/// Navigation utilities for list-based views.
///
/// In GPUI, navigation is handled through delegates (not ListState directly).
/// This module provides helper functions for common navigation patterns.
///
/// ## Usage Pattern
///
/// Delegates should implement their own selection logic (e.g., `select_down()`,
/// `select_up()`, `select_right()`, etc.) and then use `scroll_to_item()` to
/// scroll the view.
///
/// Example:
/// ```ignore
/// list_state.update(cx, |list_state, cx| {
///     let delegate = list_state.delegate_mut();
///     delegate.select_down(); // Delegate handles selection
///     if let Some(idx) = delegate.selected_index() {
///         NavigationController::scroll_to_item(
///             list_state,
///             IndexPath::new(idx),
///             window,
///             cx,
///         );
///     }
///     cx.notify();
/// });
/// ```
pub struct NavigationController;

impl NavigationController {
    /// Scroll to the specified item index in a list.
    ///
    /// This is a helper that delegates call after updating their selected index.
    pub fn scroll_to_item<D: ListDelegate>(
        list_state: &mut ListState<D>,
        index: IndexPath,
        window: &mut Window,
        cx: &mut Context<ListState<D>>,
    ) {
        list_state.scroll_to_item(index, ScrollStrategy::Top, window, cx);
    }

    /// Scroll to an item by flat index (single-section lists).
    pub fn scroll_to_index<D: ListDelegate>(
        list_state: &mut ListState<D>,
        index: usize,
        window: &mut Window,
        cx: &mut Context<ListState<D>>,
    ) {
        Self::scroll_to_item(list_state, IndexPath::new(index), window, cx);
    }

    /// Scroll to an item by section and row (multi-section lists).
    pub fn scroll_to_section_row<D: ListDelegate>(
        list_state: &mut ListState<D>,
        section: usize,
        row: usize,
        window: &mut Window,
        cx: &mut Context<ListState<D>>,
    ) {
        Self::scroll_to_item(list_state, IndexPath::new(row).section(section), window, cx);
    }
}
