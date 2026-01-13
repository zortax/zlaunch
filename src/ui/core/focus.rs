use gpui::{App, Context, FocusHandle, Focusable, Subscription, Window};

/// Manages focus for a view with automatic blur handling
pub struct FocusManager {
    focus_handle: FocusHandle,
    blur_subscription: Option<Subscription>,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new<T>(cx: &mut Context<T>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            blur_subscription: None,
        }
    }

    /// Get the focus handle
    pub fn handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Subscribe to blur events (when focus is lost)
    pub fn on_blur<V, F>(
        &mut self,
        window: &mut Window,
        cx: &mut Context<V>,
        callback: F,
    ) -> &mut Self
    where
        V: 'static,
        F: Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
    {
        let handle = self.focus_handle.clone();
        self.blur_subscription = Some(cx.on_blur(&handle, window, callback));
        self
    }

    /// Request focus for this view
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus_handle, cx);
    }

    /// Check if this view has focus
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
}

impl Focusable for FocusManager {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
