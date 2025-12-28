//! AI response view for displaying streaming responses.

use crate::ui::markdown::render_markdown_with_id;
use crate::ui::theme::theme;
use gpui::{App, Div, ElementId, SharedString, Window, div, prelude::*};
use gpui_component::scroll::ScrollableElement;
use llm::chat::ChatMessage;

/// View for displaying AI response with streaming support.
#[derive(Clone)]
pub struct AiResponseView {
    /// The messages exchanged between the user and the AI.
    messages: Vec<ChatMessage>,
    /// Whether streaming is in progress
    is_streaming: bool,
    /// Error message if the request failed
    error: Option<String>,
}

impl AiResponseView {
    /// Create a new AI response view for a query.
    pub fn new(query: String) -> Self {
        Self {
            messages: vec![
                ChatMessage::user().content(query).build(),
                ChatMessage::assistant().content("").build(),
            ],
            is_streaming: true,
            error: None,
        }
    }

    /// Append a token to the latest assistant response.
    pub fn append_token(&mut self, token: &str) {
        self.messages.last_mut().unwrap().content.push_str(token);
    }

    /// Mark streaming as complete.
    pub fn finish_streaming(&mut self) {
        self.is_streaming = false;
    }

    /// Add a new user message.
    pub fn add_user_message(&mut self, message: String) {
        self.messages
            .push(ChatMessage::user().content(message).build());
        self.messages
            .push(ChatMessage::assistant().content("").build());
    }

    /// Set an error message.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.is_streaming = false;
    }

    /// Get the current messages.
    pub fn messages(&self) -> &Vec<ChatMessage> {
        &self.messages
    }

    /// Check if streaming is in progress.
    pub fn is_streaming(&self) -> bool {
        self.is_streaming
    }

    /// Check if there's an error.
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Render the AI response view.
    pub fn render(&self, window: &mut Window, cx: &mut App) -> Div {
        let t = theme();

        let mut container = div().w_full().h_full().flex().flex_col().gap_3().p_0();

        // Show response or error
        if let Some(error) = &self.error {
            container = container.child(
                div()
                    .id("ai-error-scroll")
                    .flex_1()
                    .w_full()
                    .p_4()
                    .overflow_y_scrollbar()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .p_3()
                            .bg(t.ai.error_background)
                            .rounded_md()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(t.ai.error_title_color)
                                    .child(SharedString::from("Error")),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(t.ai.error_message_color)
                                    .child(SharedString::from(error.clone())),
                            ),
                    ),
            );
        } else {
            container = container.child(self.render_chat_messages(window, cx, &t));
        }

        container
    }

    /// Render all chat messages with user bubbles and plain assistant text.
    fn render_chat_messages(
        &self,
        window: &mut Window,
        cx: &mut App,
        t: &crate::ui::theme::LauncherTheme,
    ) -> impl IntoElement {
        let mut messages_container = div().flex().flex_col().gap(t.ai.message_gap).w_full().p_4();

        for (i, msg) in self.messages.iter().enumerate() {
            let is_last = i == self.messages.len() - 1;
            let is_streaming_msg = is_last && self.is_streaming;

            match msg.role {
                llm::chat::ChatRole::User => {
                    messages_container =
                        messages_container.child(self.render_user_bubble(i, &msg.content, t));
                }
                llm::chat::ChatRole::Assistant => {
                    messages_container = messages_container.child(self.render_assistant_message(
                        i,
                        &msg.content,
                        is_streaming_msg,
                        window,
                        cx,
                        t,
                    ));
                }
            }
        }

        div()
            .id("ai-response-scroll")
            .flex_1()
            .w_full()
            .overflow_y_scrollbar()
            .child(messages_container)
    }

    /// Render a user message as a right-aligned bubble.
    fn render_user_bubble(
        &self,
        index: usize,
        content: &str,
        t: &crate::ui::theme::LauncherTheme,
    ) -> impl IntoElement {
        // Use a flex-shrink-0 wrapper to prevent layout shifts
        div()
            .id(ElementId::Name(format!("user-msg-{}", index).into()))
            .w_full()
            .flex_shrink_0()
            .flex()
            .flex_row()
            .justify_start()
            .overflow_hidden()
            .child(
                div()
                    .max_w_full()
                    .px(t.ai.user_bubble_padding_x)
                    .py(t.ai.user_bubble_padding_y)
                    .bg(t.item_background_selected) // Use selection color
                    .rounded(t.ai.user_bubble_border_radius)
                    .text_sm()
                    .text_color(t.item_title_color) // Use title color
                    .whitespace_normal()
                    .child(SharedString::from(content.to_string())),
            )
    }

    /// Render an assistant message as plain markdown.
    fn render_assistant_message(
        &self,
        index: usize,
        content: &str,
        is_streaming: bool,
        window: &mut Window,
        cx: &mut App,
        t: &crate::ui::theme::LauncherTheme,
    ) -> impl IntoElement {
        // Use a consistent wrapper to prevent layout shifts
        let wrapper = div()
            .id(ElementId::Name(format!("assistant-msg-{}", index).into()))
            .w_full()
            .flex_shrink_0();

        if content.is_empty() && is_streaming {
            // Show "Thinking..." placeholder
            wrapper.child(
                div()
                    .text_sm()
                    .italic()
                    .text_color(t.item_description_color)
                    .child(SharedString::from("Thinking...")),
            )
        } else {
            // Render markdown content with optional streaming cursor
            let display_content = if is_streaming {
                format!("{} \u{258C}", content) // Add cursor character
            } else {
                content.to_string()
            };

            let markdown_id = format!("ai-markdown-{}", index);
            wrapper.child(render_markdown_with_id(
                markdown_id,
                &display_content,
                window,
                cx,
            ))
        }
    }
}
