//! AI mode handler for the launcher.
//!
//! This module encapsulates all AI-specific functionality:
//! - Managing the AI response view
//! - Coordinating with the AI streaming module
//! - Handling token updates and rendering
//!
//! The launcher just delegates to this handler instead of
//! containing AI logic directly.

use crate::ai;
use crate::ui::views::AiResponseView;
use flume::Receiver;
use gpui::{AsyncApp, Context, Task, WeakEntity, Window};
use gpui_component::input::InputState;

/// Handler for AI response mode.
///
/// This encapsulates all state and logic for the AI mode,
/// keeping it separate from the main launcher logic.
pub struct AiModeHandler {
    /// The AI response view
    view: AiResponseView,
    /// Task for polling the streaming channel
    /// (stored to keep it alive, but never read)
    stream_task: Task<()>,
}

impl AiModeHandler {
    /// Create a new AI mode handler and start streaming.
    ///
    /// # Arguments
    /// * `query` - The user's query to send to the AI
    /// * `launcher_entity` - Weak reference to the launcher for updates
    /// * `cx` - GPUI context
    ///
    /// # Returns
    /// - `Some(handler)` if streaming started successfully
    /// - `None` if AI is unavailable or streaming failed to start
    pub fn new<T>(
        query: String,
        launcher_entity: WeakEntity<T>,
        cx: &mut Context<T>,
    ) -> Option<Self>
    where
        T: AiModeAccess + 'static,
    {
        // Create the view
        let view = AiResponseView::new(query.clone());

        // Start streaming from the AI module
        let rx = ai::spawn_stream(view.messages().clone())?;

        // Create task to poll the channel
        let stream_task = Self::spawn_polling_task(rx, launcher_entity, cx);

        Some(Self { view, stream_task })
    }

    /// Send a new user message. Cancels the current streaming task.
    pub fn send_message<T>(
        &mut self,
        message: String,
        launcher_entity: WeakEntity<T>,
        cx: &mut Context<T>,
    ) where
        T: AiModeAccess + 'static,
    {
        // Abort the current task by replacing it with a ready task
        self.stream_task = Task::ready(());

        self.view.finish_streaming();
        self.view.add_user_message(message);

        // Start streaming from the AI module
        if let Some(rx) = ai::spawn_stream(self.view.messages().clone()) {
            // Create task to poll the channel
            self.stream_task = Self::spawn_polling_task(rx, launcher_entity, cx);
        }
    }

    /// Spawn a task that polls the streaming channel and updates the view.
    fn spawn_polling_task<T>(
        rx: Receiver<Result<String, String>>,
        launcher_entity: WeakEntity<T>,
        cx: &mut Context<T>,
    ) -> Task<()>
    where
        T: AiModeAccess + 'static,
    {
        cx.spawn(async move |_entity: WeakEntity<T>, cx: &mut AsyncApp| {
            while let Ok(msg) = rx.recv_async().await {
                let is_complete = matches!(msg, Ok(ref s) if s.is_empty());
                let is_error = msg.is_err();

                let _ = cx.update(|cx| {
                    if let Some(launcher) = launcher_entity.upgrade() {
                        launcher.update(cx, |launcher, cx| {
                            // Call the update method on the launcher
                            // The launcher will forward this to the mode handler
                            Self::update_view_with_token(launcher, msg, cx);
                        });
                    }
                });

                if is_complete || is_error {
                    break;
                }
            }
        })
    }

    /// Update the AI view with a token or error.
    ///
    /// This is called from the polling task through the launcher.
    /// We need access to self through the launcher's update context.
    fn update_view_with_token<T>(launcher: &mut T, msg: Result<String, String>, cx: &mut Context<T>)
    where
        T: AiModeAccess + 'static,
    {
        if let Some(handler) = launcher.ai_mode_handler_mut() {
            match msg {
                Ok(token) => {
                    if !token.is_empty() {
                        handler.view.append_token(&token);
                    } else {
                        // Empty token means streaming complete
                        handler.view.finish_streaming();
                    }
                }
                Err(error) => {
                    handler.view.set_error(error);
                }
            }
            cx.notify();
        }
    }

    /// Get a reference to the AI response view for rendering.
    pub fn view(&self) -> &AiResponseView {
        &self.view
    }

    /// Update input placeholder when entering AI mode.
    pub fn setup_input(
        input_state: &mut InputState,
        window: &mut Window,
        cx: &mut Context<InputState>,
    ) {
        input_state.set_value("", window, cx);
        input_state.set_placeholder("Send a new prompt or stop response...", window, cx);
    }

    /// Clear the input value when sending a new prompt.
    pub fn clear_input(
        input_state: &mut InputState,
        window: &mut Window,
        cx: &mut Context<InputState>,
    ) {
        input_state.set_value("", window, cx);
    }

    /// Restore input placeholder when exiting AI mode.
    pub fn restore_input(
        input_state: &mut InputState,
        window: &mut Window,
        cx: &mut Context<InputState>,
    ) {
        input_state.set_value("", window, cx);
        input_state.set_placeholder("Search applications...", window, cx);
    }
}

/// Trait for types that can provide access to the AI mode handler.
///
/// This allows the polling task to update the handler through the launcher.
pub trait AiModeAccess {
    fn ai_mode_handler_mut(&mut self) -> Option<&mut AiModeHandler>;
}
