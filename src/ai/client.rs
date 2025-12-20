//! LLM API client for streaming AI responses.

use anyhow::{Context, Result};
use futures::Stream;
use futures::stream::StreamExt;
use llm::LLMProvider;
use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::ChatMessage;
use std::env;
use std::pin::Pin;

/// LLM client for AI queries.
pub struct LLMClient {
    llm: Box<dyn LLMProvider>,
}

impl LLMClient {
    /// Create a new LLM client.
    /// Returns None if no valid API_KEY environment variable is set.
    pub fn new() -> Option<Self> {
        // Find the first available environment variable
        let (api_key, backend) = [
            ("GEMINI_API_KEY", LLMBackend::Google),
            ("OPENAI_API_KEY", LLMBackend::OpenAI),
            ("OPENROUTER_API_KEY", LLMBackend::OpenRouter),
        ]
        .iter()
        .find_map(|(var_name, backend)| env::var(var_name).ok().map(|key| (key, backend)))?;

        LLMBuilder::new()
            .backend(backend.clone())
            .api_key(&api_key)
            .model(match backend {
                LLMBackend::Google => "gemini-flash-latest".to_string(),
                LLMBackend::OpenAI => "gpt-5-mini".to_string(),
                LLMBackend::OpenRouter => env::var("OPENROUTER_MODEL")
                    .unwrap_or_else(|_| "google/gemini-2.5-flash".to_string()),
                _ => unreachable!(),
            })
            .max_tokens(2000)
            .temperature(0.7)
            .build()
            .ok()
            .map(|llm| Self { llm })
    }

    /// Stream a response for the given query.
    /// Returns a stream of tokens (strings).
    pub async fn stream_query(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let stream = self
            .llm
            .chat_stream(messages)
            .await
            .context("Failed to initiate streaming chat")?;

        // Convert LLMError to anyhow::Error
        let result_stream =
            stream.map(|result| result.map_err(|e| anyhow::Error::msg(e.to_string())));

        Ok(Box::pin(result_stream))
    }
}

impl Default for LLMClient {
    fn default() -> Self {
        Self::new().expect(
            "GEMINI_API_KEY, OPENAI_API_KEY or OPENROUTER_API_KEY environment variable not set",
        )
    }
}
