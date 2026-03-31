//! Anthropic API provider (Claude models).
//!
//! Uses the Messages API (/v1/messages).
//! Supports claude-3-5-sonnet, claude-3-opus, claude-3-haiku and newer.

use super::traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Anthropic API provider.
#[derive(Clone)]
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    display_name: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        let display_name = format!("Anthropic {}", model);
        AnthropicProvider {
            client: reqwest::Client::new(),
            api_key,
            model,
            display_name,
        }
    }

    pub fn from_config(config: &crate::config::AnthropicConfig) -> Self {
        Self::new(config.api_key.clone(), config.model.clone())
    }
}

// ── Anthropic request/response shapes ───────────────────────────────────────

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    model: String,
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

// ── LLMProvider implementation ───────────────────────────────────────────────

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn id(&self) -> &str {
        "anthropic"
    }

    fn supports_system_prompt(&self) -> bool {
        // Anthropic Messages API has a dedicated `system` field
        true
    }

    async fn complete(
        &self,
        system_prompt: Option<&str>,
        user_message: &str,
        config: &RequestConfig,
    ) -> Result<LLMResponse, ProviderError> {
        let body = MessagesRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: user_message.to_string(),
            }],
            system: system_prompt.map(|s| s.to_string()),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
        };

        let start = Instant::now();
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            // Anthropic uses x-api-key header, not Bearer token
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let latency_ms = start.elapsed().as_millis() as u64;

        if status == 401 {
            return Err(ProviderError::AuthError);
        }
        if status == 429 {
            return Err(ProviderError::RateLimited { retry_after_secs: 60 });
        }
        if !status.is_success() {
            let msg = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError {
                status: status.as_u16(),
                message: msg,
            });
        }

        let parsed: MessagesResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        // Collect all text blocks into a single string
        let text = parsed
            .content
            .into_iter()
            .filter(|b| b.block_type == "text")
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(LLMResponse {
            text,
            model: parsed.model,
            prompt_tokens: Some(parsed.usage.input_tokens),
            completion_tokens: Some(parsed.usage.output_tokens),
            latency_ms,
        })
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        let config = RequestConfig { temperature: 0.0, max_tokens: 5 };
        self.complete(None, "ping", &config).await?;
        Ok(())
    }
}
