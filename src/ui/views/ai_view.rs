//! AI response view for displaying streaming responses.

use crate::ui::markdown::render_markdown;
use crate::ui::theme::theme;
use gpui::{App, Div, SharedString, Window, div, prelude::*};
use gpui_component::scroll::ScrollableElement;

/// View for displaying AI response with streaming support.
#[derive(Clone)]
pub struct AiResponseView {
    /// The original query
    query: String,
    /// The accumulated response text
    response: String,
    /// Whether streaming is in progress
    is_streaming: bool,
    /// Error message if the request failed
    error: Option<String>,
}

impl AiResponseView {
    /// Create a new AI response view for a query.
    pub fn new(query: String) -> Self {
        Self {
            query,
            response: String::new(),
            is_streaming: true,
            error: None,
        }
    }

    /// Append a token to the response.
    pub fn append_token(&mut self, token: &str) {
        self.response.push_str(token);
    }

    /// Mark streaming as complete.
    pub fn finish_streaming(&mut self) {
        self.is_streaming = false;
    }

    /// Set an error message.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.is_streaming = false;
    }

    /// Get the current response text.
    pub fn response(&self) -> &str {
        &self.response
    }

    /// Get the query.
    pub fn query(&self) -> &str {
        &self.query
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
        } else if self.response.is_empty() && self.is_streaming {
            // Show loading indicator
            div()
                .id("ai-loading-scroll")
                .flex_1()
                .w_full()
                .p_4()
                .overflow_y_scrollbar()
                .child(
                    div()
                        .text_base()
                        .text_color(t.item_description_color)
                        .child(SharedString::from("Thinking...")),
                )
        } else {
            // Show response text with markdown rendering (scrollable)
            let mut response_text = self.response.clone();

            // Add cursor if streaming
            if self.is_streaming {
                response_text.push_str(" ▌");
            }

            let response_content =
                div()
                    .w_full()
                    .p_4()
                    .child(render_markdown(&response_text, window, cx));

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
