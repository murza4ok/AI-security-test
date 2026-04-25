//! Core provider abstraction.
//!
//! Every LLM backend implements `LLMProvider`. The attack engine
//! depends only on this trait, so adding a new provider requires
//! no changes to attack logic.

use async_trait::async_trait;
use thiserror::Error;

use crate::engine::session::TargetMetadata;

/// Configuration for a single completion request.
/// Passed to every `complete()` call so callers can tune behaviour per request.
#[derive(Debug, Clone)]
pub struct RequestConfig {
    /// Sampling temperature (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,
    /// Maximum tokens in the response
    pub max_tokens: u32,
}

impl Default for RequestConfig {
    fn default() -> Self {
        RequestConfig {
            temperature: 0.7,
            max_tokens: 1024,
        }
    }
}

/// A response received from an LLM provider.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LLMResponse {
    /// The model's text output
    pub text: String,
    /// Which model actually generated the response (may differ from requested)
    pub model: String,
    /// Prompt tokens used (if reported by the provider)
    pub prompt_tokens: Option<u32>,
    /// Completion tokens used (if reported by the provider)
    pub completion_tokens: Option<u32>,
    /// Wall-clock latency in milliseconds
    pub latency_ms: u64,
}

/// Errors that can occur when communicating with an LLM provider.
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Authentication failed: invalid or missing API key")]
    AuthError,

    #[error("Rate limited by provider — retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Failed to parse provider response: {0}")]
    ParseError(String),

    #[error("Provider returned an error: {status} — {message}")]
    ApiError { status: u16, message: String },

    #[error("Request timed out after {timeout_secs}s")]
    #[allow(dead_code)]
    Timeout { timeout_secs: u64 },

    #[error("Provider is not configured (missing API key or URL)")]
    NotConfigured,
}

/// The central abstraction for all LLM providers.
/// Implement this trait to add support for a new provider.
#[allow(dead_code)]
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Human-readable provider + model name for display (e.g., "OpenAI GPT-4o")
    fn name(&self) -> &str;

    /// Short identifier used in CLI and reports (e.g., "openai", "anthropic", "ollama")
    fn id(&self) -> &str;

    /// Configured model name requested for this provider instance.
    fn configured_model(&self) -> &str;

    /// Whether this provider accepts a dedicated system prompt field.
    /// The current provider layer passes system prompts natively for all
    /// built-in providers; future integrations can use this capability bit
    /// when adding non-chat transports.
    fn supports_system_prompt(&self) -> bool;

    /// Send a single completion request.
    ///
    /// `system_prompt` is optional and forwarded according to the provider's
    /// native request shape.
    async fn complete(
        &self,
        system_prompt: Option<&str>,
        user_message: &str,
        config: &RequestConfig,
    ) -> Result<LLMResponse, ProviderError>;

    /// Verify that the provider is reachable and the credentials are valid.
    /// Used by `ai-sec check` before running attacks.
    async fn health_check(&self) -> Result<(), ProviderError>;

    /// Return external target metadata, if this provider talks to a concrete app target.
    fn target_metadata(&self) -> Option<TargetMetadata> {
        None
    }
}
