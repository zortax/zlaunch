//! Gemini API client for streaming AI responses.

use anyhow::{Context, Result};
use futures::Stream;
use futures::stream::StreamExt;
use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::ChatMessage;
use std::env;
use std::pin::Pin;

/// Gemini client for AI queries.
pub struct GeminiClient {
    api_key: String,
}

impl GeminiClient {
    /// Create a new Gemini client.
    /// Returns None if GEMINI_API_KEY environment variable is not set.
    pub fn new() -> Option<Self> {
        env::var("GEMINI_API_KEY")
            .ok()
            .map(|api_key| Self { api_key })
    }

    /// Check if the Gemini client is available (API key is set).
    pub fn is_available() -> bool {
        env::var("GEMINI_API_KEY").is_ok()
    }

    /// Stream a response for the given query.
    /// Returns a stream of tokens (strings).
    pub async fn stream_query(
        &self,
        query: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let llm = LLMBuilder::new()
            .backend(LLMBackend::Google)
            .api_key(&self.api_key)
            .model("gemini-flash-latest")
            .max_tokens(2000)
            .temperature(0.7)
            .build()
            .context("Failed to build Gemini client")?;

        let messages = vec![ChatMessage::user().content(query).build()];

        let stream = llm
            .chat_stream(&messages)
            .await
            .context("Failed to initiate streaming chat")?;

        // Convert LLMError to anyhow::Error
        let result_stream =
            stream.map(|result| result.map_err(|e| anyhow::Error::msg(e.to_string())));

        Ok(Box::pin(result_stream))
    }
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self::new().expect("GEMINI_API_KEY environment variable not set")
    }
}
