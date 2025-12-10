//! AI integration module for zlaunch.
//!
//! Provides integration with Gemini API for answering user queries.

pub mod client;
pub mod item;
pub mod streaming;

pub use client::GeminiClient;
pub use item::AiItem;
pub use streaming::spawn_stream;
