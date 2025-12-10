use std::sync::Arc;

/// Type alias for confirm callbacks to reduce complexity
type ConfirmCallback<T> = Arc<dyn Fn(&T) + Send + Sync>;

/// Type alias for cancel callbacks to reduce complexity
type CancelCallback = Arc<dyn Fn() + Send + Sync>;

/// Common state and behavior for all list delegates.
///
/// This provides a base implementation that can be composed into specific delegates,
/// eliminating the duplication seen in the old code where each delegate (ItemListDelegate,
/// EmojiGridDelegate, ClipboardListDelegate) had nearly identical selection/callback logic.
///
/// # Type Parameters
/// - `T`: The item type being displayed in the list
pub struct BaseDelegate<T: Clone> {
    /// All available items
    items: Vec<T>,
    /// Filtered/visible item indices
    filtered_indices: Vec<usize>,
    /// Currently selected index (in filtered items)
    selected_index: Option<usize>,
    /// Current search query
    query: String,
    /// Callback when an item is confirmed/selected
    on_confirm: Option<ConfirmCallback<T>>,
    /// Callback when the list is cancelled
    on_cancel: Option<CancelCallback>,
}

impl<T: Clone> BaseDelegate<T> {
    /// Create a new base delegate with the given items
    pub fn new(items: Vec<T>) -> Self {
        let len = items.len();
        let filtered_indices: Vec<usize> = (0..len).collect();

        Self {
            items,
            filtered_indices,
            selected_index: if len > 0 { Some(0) } else { None },
            query: String::new(),
            on_confirm: None,
            on_cancel: None,
        }
    }

    /// Set the confirm callback
    pub fn set_on_confirm(&mut self, callback: impl Fn(&T) + Send + Sync + 'static) {
        self.on_confirm = Some(Arc::new(callback));
    }

    /// Set the cancel callback
    pub fn set_on_cancel(&mut self, callback: impl Fn() + Send + Sync + 'static) {
        self.on_cancel = Some(Arc::new(callback));
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Set the selected index
    pub fn set_selected(&mut self, index: usize) {
        if index < self.filtered_count() {
            self.selected_index = Some(index);
        }
    }

    /// Set the selected index without bounds checking
    /// (for use by delegates with dynamic items that extend beyond filtered_count)
    pub fn set_selected_unchecked(&mut self, index: usize) {
        self.selected_index = Some(index);
    }

    /// Get the number of filtered items
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Get the current query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Set the query (caller should then call filter())
    pub fn set_query(&mut self, query: String) {
        self.query = query;
    }

    /// Clear the query and reset filtering
    pub fn clear_query(&mut self) {
        self.query.clear();
        self.reset_filter();
    }

    /// Reset to show all items
    pub fn reset_filter(&mut self) {
        self.filtered_indices = (0..self.items.len()).collect();
        if !self.filtered_indices.is_empty() {
            self.selected_index = Some(0);
        } else {
            self.selected_index = None;
        }
    }

    /// Apply filtered indices (used after filtering on a background thread)
    pub fn apply_filtered_indices(&mut self, indices: Vec<usize>) {
        self.filtered_indices = indices;
        // Reset selection to first item
        if !self.filtered_indices.is_empty() {
            self.selected_index = Some(0);
        } else {
            self.selected_index = None;
        }
    }

    /// Get an item by filtered index
    pub fn get_filtered_item(&self, filtered_index: usize) -> Option<&T> {
        self.filtered_indices
            .get(filtered_index)
            .and_then(|&item_idx| self.items.get(item_idx))
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&T> {
        self.selected_index
            .and_then(|idx| self.get_filtered_item(idx))
    }

    /// Execute the confirm callback
    pub fn do_confirm(&self) {
        if let Some(item) = self.selected_item()
            && let Some(ref callback) = self.on_confirm
        {
            callback(item);
        }
    }

    /// Execute the cancel callback
    pub fn do_cancel(&self) {
        if let Some(ref callback) = self.on_cancel {
            callback();
        }
    }

    /// Move selection down (with wrapping)
    pub fn select_down(&mut self) {
        let count = self.filtered_count();
        if count == 0 {
            return;
        }

        let current = self.selected_index.unwrap_or(0);
        let next = if current + 1 >= count { 0 } else { current + 1 };
        self.selected_index = Some(next);
    }

    /// Move selection up (with wrapping)
    pub fn select_up(&mut self) {
        let count = self.filtered_count();
        if count == 0 {
            return;
        }

        let current = self.selected_index.unwrap_or(0);
        let prev = if current == 0 { count - 1 } else { current - 1 };
        self.selected_index = Some(prev);
    }

    /// Get all items (for external filtering)
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Get the raw filtered indices
    pub fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_delegate_creation() {
        let items = vec!["item1", "item2", "item3"];
        let delegate = BaseDelegate::new(items);

        assert_eq!(delegate.filtered_count(), 3);
        assert_eq!(delegate.selected_index(), Some(0));
    }

    #[test]
    fn test_selection_navigation() {
        let items = vec!["a", "b", "c"];
        let mut delegate = BaseDelegate::new(items);

        assert_eq!(delegate.selected_index(), Some(0));

        delegate.select_down();
        assert_eq!(delegate.selected_index(), Some(1));

        delegate.select_down();
        assert_eq!(delegate.selected_index(), Some(2));

        delegate.select_down(); // Wraps to 0
        assert_eq!(delegate.selected_index(), Some(0));

        delegate.select_up(); // Wraps to 2
        assert_eq!(delegate.selected_index(), Some(2));
    }
}
