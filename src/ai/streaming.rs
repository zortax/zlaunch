//! AI streaming coordinator.
//!
//! This module handles the low-level details of streaming AI responses:
//! - Spawning Tokio runtime
//! - Managing async streams
//! - Converting async tokens to sync channel for GPUI
//!
//! The UI layer just receives tokens through a channel without dealing
//! with async complexity.

use super::LLMClient;
use flume::Receiver;
use llm::chat::ChatMessage;
use tokio::sync::OnceCell;

static CLIENT: OnceCell<LLMClient> = OnceCell::const_new();

/// Spawn an AI streaming task and return a channel receiver for tokens.
///
/// This function handles all the async/tokio complexity internally:
/// - Creates a Tokio runtime
/// - Spawns a thread
/// - Streams tokens from the AI client
/// - Sends tokens through a channel
///
/// The caller just needs to poll the receiver in their event loop.
///
/// # Returns
/// A receiver that yields:
/// - `Ok(token)` for each token received
/// - `Ok("")` when streaming completes successfully
/// - `Err(error)` if an error occurs
pub fn spawn_stream(messages: Vec<ChatMessage>) -> Option<Receiver<Result<String, String>>> {
    // Create channel for communication between Tokio thread and caller
    let (tx, rx) = flume::unbounded::<Result<String, String>>();

    // Spawn Tokio thread for LLM request
    std::thread::spawn(move || {
        // Create a single-threaded Tokio runtime
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            // Create LLM client if it doesn't exist
            let client = CLIENT
                .get_or_init(async || LLMClient::new().expect("Failed to create LLM client"))
                .await;

            // Start streaming
            let stream_result = client.stream_query(&messages).await;

            match stream_result {
                Ok(mut stream) => {
                    use futures::StreamExt;

                    // Process tokens as they arrive
                    while let Some(token_result) = stream.next().await {
                        match token_result {
                            Ok(token) => {
                                // Send token through channel
                                if tx.send(Ok(token)).is_err() {
                                    break; // Channel closed, stop streaming
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(format!("Stream error: {}", e)));
                                break;
                            }
                        }
                    }
                    // Send completion signal (empty Ok)
                    let _ = tx.send(Ok(String::new()));
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to connect: {}", e)));
                }
            }
        });
    });

    Some(rx)
}
