//! LLM API client for streaming AI responses.

use anyhow::{Result, anyhow};
use futures::Stream;
use futures::stream::{StreamExt, once};
use llm::LLMProvider;
use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::ChatMessage;
use std::env;
use std::pin::Pin;

fn get_keys() -> Option<(String, LLMBackend)> {
    [
        ("OLLAMA_URL", LLMBackend::Ollama),
        ("GEMINI_API_KEY", LLMBackend::Google),
        ("OPENAI_API_KEY", LLMBackend::OpenAI),
        ("OPENROUTER_API_KEY", LLMBackend::OpenRouter),
    ]
    .iter()
    .find_map(|(var_name, backend)| {
        env::var(var_name)
            .ok()
            .map(|value| (value, backend.clone()))
    })
}

/// LLM client for AI queries.
pub struct LLMClient {
    llm: Box<dyn LLMProvider>,
    backend: LLMBackend,
}

impl LLMClient {
    /// Create a new LLM client.
    /// Returns None if no valid API_KEY environment variable is set.
    pub fn new() -> Option<Self> {
        let (api_key, backend) = get_keys()?;

        let mut builder = LLMBuilder::new().backend(backend.clone());

        builder = match backend {
            LLMBackend::Ollama => builder.base_url(&api_key),
            _ => builder.api_key(&api_key),
        };

        let llm = builder
            .model(match backend {
                LLMBackend::Ollama => {
                    env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2:latest".to_string())
                }
                LLMBackend::Google => "gemini-flash-latest".to_string(),
                LLMBackend::OpenAI => "gpt-5-mini".to_string(),
                LLMBackend::OpenRouter => env::var("OPENROUTER_MODEL")
                    .unwrap_or_else(|_| "google/gemini-2.5-flash".to_string()),
                _ => unreachable!(),
            })
            .max_tokens(2000)
            .temperature(0.7)
            .build()
            .ok()?;

        Some(Self {
            llm,
            backend: backend.clone(),
        })
    }

    /// Return true if any LLM is configured.
    pub fn is_configured() -> bool {
        get_keys().is_some()
    }

    /// Stream a response for the given query.
    /// Returns a stream of tokens (strings).
    /// Ollama doesnt support streaming for some reason so it uses normal chat().
    pub async fn stream_query(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        if self.backend == LLMBackend::Ollama {
            let response = self.llm.chat(messages).await?;

            let text = response
                .text()
                .ok_or_else(|| anyhow!("LLM response missing text"))?;

            let result = once(async move { Ok(text) });

            Ok(Box::pin(result))
        } else {
            let stream = self.llm.chat_stream(messages).await?;

            // Convert LLMError to anyhow::Error
            let result_stream =
                stream.map(|result| result.map_err(|e| anyhow::Error::msg(e.to_string())));

            Ok(Box::pin(result_stream))
        }
    }

    // Keeping this here bc if llm crate adds streaming for ollama we can probably just uncomment this and remove above code.

    // pub async fn stream_query(
    //     &self,
    //     messages: &[ChatMessage],
    // ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
    //     let stream = self
    //         .llm
    //         .chat_stream(messages)
    //         .await
    //         .context("Failed to initiate streaming chat")?;

    //     // Convert LLMError to anyhow::Error
    //     let result_stream =
    //         stream.map(|result| result.map_err(|e| anyhow::Error::msg(e.to_string())));

    //     Ok(Box::pin(result_stream))
    // }
}

impl Default for LLMClient {
    fn default() -> Self {
        Self::new().expect(
            "GEMINI_API_KEY, OPENAI_API_KEY, OPENROUTER_API_KEY or OLLAMA_URL environment variable not set",
        )
    }
}
