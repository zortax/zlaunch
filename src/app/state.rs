use crate::ipc::IpcServerHandle;
use crate::items::{ApplicationItem, ListItem};
use gpui_kit::WindowHandle;
use gpui_kit::component::Root;

/// Context for a single view in the navigation stack.
/// Each view has its own items, filter state, and selection.
#[derive(Clone)]
pub struct ViewContext {
    pub title: String,
    pub items: Vec<ListItem>,
    pub filtered_indices: Vec<usize>,
    pub query: String,
    pub selected_index: Option<usize>,
}

impl ViewContext {
    /// Create a new view context with the given items.
    pub fn new(title: impl Into<String>, items: Vec<ListItem>) -> Self {
        let len = items.len();
        Self {
            title: title.into(),
            items,
            filtered_indices: (0..len).collect(),
            query: String::new(),
            selected_index: if len > 0 { Some(0) } else { None },
        }
    }

    /// Reset the view to its initial state.
    pub fn reset(&mut self) {
        let len = self.items.len();
        self.filtered_indices = (0..len).collect();
        self.query.clear();
        self.selected_index = if len > 0 { Some(0) } else { None };
    }
}

/// Global application state for the launcher daemon.
pub struct AppState {
    /// All loaded applications
    pub applications: Vec<ApplicationItem>,
    /// Navigation stack for submenu support (future)
    pub view_stack: Vec<ViewContext>,
    /// IPC server handle for receiving commands
    pub ipc_server: Option<IpcServerHandle>,
    /// Current window handle if visible
    pub window_handle: Option<WindowHandle<Root>>,
    /// Whether the window is currently visible
    pub window_visible: bool,
}

impl AppState {
    /// Create new application state with loaded applications.
    pub fn new(applications: Vec<ApplicationItem>, ipc_server: Option<IpcServerHandle>) -> Self {
        Self {
            applications,
            view_stack: Vec::new(),
            ipc_server,
            window_handle: None,
            window_visible: false,
        }
    }

    /// Get the main view items (applications as ListItems).
    pub fn main_view_items(&self) -> Vec<ListItem> {
        self.applications
            .iter()
            .cloned()
            .map(ListItem::Application)
            .collect()
    }

    /// Push a new view onto the navigation stack.
    pub fn push_view(&mut self, view: ViewContext) {
        self.view_stack.push(view);
    }

    /// Pop the top view from the navigation stack.
    pub fn pop_view(&mut self) -> Option<ViewContext> {
        self.view_stack.pop()
    }

    /// Check if we're in a submenu (view stack is not empty).
    pub fn in_submenu(&self) -> bool {
        !self.view_stack.is_empty()
    }

    /// Get the current view context (top of stack or None for main view).
    pub fn current_view(&self) -> Option<&ViewContext> {
        self.view_stack.last()
    }

    /// Get mutable reference to current view context.
    pub fn current_view_mut(&mut self) -> Option<&mut ViewContext> {
        self.view_stack.last_mut()
    }
}
