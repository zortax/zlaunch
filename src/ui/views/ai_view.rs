//! AI response view for displaying streaming responses.

use crate::ui::markdown::render_markdown;
use crate::ui::theme::theme;
use gpui::{App, Div, SharedString, Window, div, prelude::*};
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

        let container = div().w_full().h_full().flex().flex_col().gap_3().p_0();

        // Show response or error
        let content = if let Some(error) = &self.error {
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
                )
        } else {
            // Show response text with markdown rendering (scrollable)
            let mut full_content = String::new();

            for (i, msg) in self.messages.iter().enumerate() {
                if i > 0 {
                    full_content.push_str("\n\n");
                }

                let role_prefix = match msg.role {
                    llm::chat::ChatRole::User => "**User:** ",
                    llm::chat::ChatRole::Assistant => "**Assistant:** ",
                };

                full_content.push_str(role_prefix);

                let is_last = i == self.messages.len() - 1;
                if is_last && self.is_streaming && msg.content.is_empty() {
                    full_content.push_str("_Thinking..._");
                } else {
                    full_content.push_str(&msg.content);

                    // Add cursor if streaming and this is the last message
                    if self.is_streaming && is_last {
                        full_content.push_str(" ▌");
                    }
                }
            }

            let response_content =
                div()
                    .w_full()
                    .p_4()
                    .child(render_markdown(&full_content, window, cx));

            div()
                .id("ai-response-scroll")
                .flex_1()
                .w_full()
                .overflow_y_scrollbar()
                .child(response_content)
        };

        container.child(content)
    }
}
