//! OpenAI API provider.
//!
//! Supports GPT-4o, GPT-4-turbo, GPT-3.5-turbo and any OpenAI-compatible
//! endpoint (Azure OpenAI, local proxies) via `base_url` override.
//! Handles 429 rate-limit responses with a configurable retry delay.

use super::{
    build_http_client, map_transport_error, retry_provider_call,
    traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig},
    RetrySettings,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// OpenAI provider instance. Cheap to clone — the inner reqwest::Client
/// is already Arc-based and reuses the connection pool.
#[derive(Clone)]
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
    display_name: String,
    timeout: Duration,
    retry_settings: RetrySettings,
}

impl OpenAIProvider {
    /// Construct a new OpenAI provider from explicit parameters.
    /// Use `from_config()` for the normal startup path.
    pub fn new(
        api_key: String,
        model: String,
        base_url: String,
        timeout: Duration,
        retry_settings: RetrySettings,
    ) -> Self {
        let display_name = format!("OpenAI {}", model);
        OpenAIProvider {
            client: build_http_client(timeout),
            api_key,
            model,
            base_url,
            display_name,
            timeout,
            retry_settings,
        }
    }

    /// Construct from the application config.
    pub fn from_config(
        config: &crate::config::OpenAIConfig,
        timeout: Duration,
        retry_settings: RetrySettings,
    ) -> Self {
        Self::new(
            config.api_key.clone(),
            config.model.clone(),
            config.base_url.clone(),
            timeout,
            retry_settings,
        )
    }
}

// ── OpenAI request/response shapes ──────────────────────────────────────────

/// A single message in the OpenAI chat completions format.
#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Request body for POST /v1/chat/completions
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

/// Top-level response from the completions endpoint
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
    model: String,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

// ── LLMProvider implementation ───────────────────────────────────────────────

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn id(&self) -> &str {
        "openai"
    }

    fn configured_model(&self) -> &str {
        &self.model
    }

    fn supports_system_prompt(&self) -> bool {
        // OpenAI chat completions supports the "system" role natively
        true
    }

    async fn complete(
        &self,
        system_prompt: Option<&str>,
        user_message: &str,
        config: &RequestConfig,
    ) -> Result<LLMResponse, ProviderError> {
        retry_provider_call(self.retry_settings, || async {
            // Build message list; add system message only if a prompt was provided
            let mut messages: Vec<ChatMessage> = Vec::new();
            if let Some(system) = system_prompt {
                messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                });
            }
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            });

            let body = ChatRequest {
                model: self.model.clone(),
                messages,
                temperature: config.temperature,
                max_tokens: config.max_tokens,
            };

            let url = format!("{}/chat/completions", self.base_url);
            let start = Instant::now();

            let response = self
                .client
                .post(&url)
                .bearer_auth(&self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(|e| map_transport_error(e, self.timeout))?;

            let status = response.status();
            let latency_ms = start.elapsed().as_millis() as u64;

            if status == 401 {
                return Err(ProviderError::AuthError);
            }
            if status == 429 {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(60);
                return Err(ProviderError::RateLimited {
                    retry_after_secs: retry_after,
                });
            }
            if !status.is_success() {
                let msg = response.text().await.unwrap_or_default();
                return Err(ProviderError::ApiError {
                    status: status.as_u16(),
                    message: msg,
                });
            }

            let parsed: ChatResponse = response
                .json()
                .await
                .map_err(|e| ProviderError::ParseError(e.to_string()))?;

            let text = parsed
                .choices
                .into_iter()
                .next()
                .map(|c| c.message.content)
                .unwrap_or_default();

            Ok(LLMResponse {
                text,
                model: parsed.model,
                prompt_tokens: parsed.usage.as_ref().map(|u| u.prompt_tokens),
                completion_tokens: parsed.usage.as_ref().map(|u| u.completion_tokens),
                latency_ms,
            })
        })
        .await
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        // Send a minimal request to verify credentials and connectivity
        let config = RequestConfig {
            temperature: 0.0,
            max_tokens: 5,
        };
        self.complete(None, "ping", &config).await?;
        Ok(())
    }
}
