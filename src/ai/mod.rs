//! AI integration module for zlaunch.
//!
//! Provides integration with LLM API for answering user queries.

pub mod client;
pub mod streaming;

pub use client::LLMClient;
pub use streaming::spawn_stream;
