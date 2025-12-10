use gpui::{Context, Entity, Subscription, Window};
use gpui_component::input::{InputEvent, InputState};

/// Manages input handling and query updates for views
/// This consolidates input subscription logic that was scattered across the old code
pub struct InputHandler<V> {
    input_state: Entity<InputState>,
    subscription: Option<Subscription>,
    _phantom: std::marker::PhantomData<V>,
}

impl<V: 'static> InputHandler<V> {
    /// Create a new input handler
    pub fn new(input_state: Entity<InputState>) -> Self {
        Self {
            input_state,
            subscription: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the input state entity
    pub fn input_state(&self) -> &Entity<InputState> {
        &self.input_state
    }

    /// Subscribe to input changes with a callback
    ///
    /// The callback receives the view, the input text, and the context.
    /// Window access is available through context methods if needed.
    pub fn subscribe<F>(
        &mut self,
        cx: &mut Context<V>,
        mut on_change: F,
    ) -> &mut Self
    where
        F: FnMut(&mut V, String, &mut Context<V>) + 'static,
    {
        let input_state = self.input_state.clone();
        let input_state_for_read = input_state.clone();
        self.subscription = Some(cx.subscribe(&input_state, move |view, _input, event, cx| {
            if let InputEvent::Change = event {
                let text = input_state_for_read.read(cx).value().to_string();
                on_change(view, text, cx);
            }
        }));
        self
    }

    /// Set the input placeholder
    pub fn set_placeholder<T>(&self, placeholder: &str, window: &mut Window, cx: &mut Context<T>) {
        let placeholder = placeholder.to_string();
        self.input_state.update(cx, move |input, cx| {
            input.set_placeholder(&placeholder, window, cx);
        });
    }

    /// Set the input value
    pub fn set_value<T>(&self, value: &str, window: &mut Window, cx: &mut Context<T>) {
        let value = value.to_string();
        self.input_state.update(cx, move |input, cx| {
            input.set_value(&value, window, cx);
        });
    }

    /// Get the current input value
    pub fn value<T>(&self, cx: &Context<T>) -> String {
        self.input_state.read(cx).value().to_string()
    }

    /// Clear the input
    pub fn clear<T>(&self, window: &mut Window, cx: &mut Context<T>) {
        self.set_value("", window, cx);
    }

    /// Focus the input
    pub fn focus<T>(&self, window: &mut Window, cx: &mut Context<T>) {
        self.input_state.update(cx, |input, cx| {
            input.focus(window, cx);
        });
    }
}
