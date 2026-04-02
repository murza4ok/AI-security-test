//! DeepSeek API provider.
//!
//! DeepSeek uses an OpenAI-compatible API, so we reuse the same
//! request/response structures but point at api.deepseek.com.

use super::traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// DeepSeek provider. Wraps an OpenAI-compatible endpoint.
#[derive(Clone)]
pub struct DeepSeekProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
    display_name: String,
}

impl DeepSeekProvider {
    pub fn new(api_key: String, model: String, base_url: String) -> Self {
        let display_name = format!("DeepSeek {}", model);
        DeepSeekProvider {
            client: reqwest::Client::new(),
            api_key,
            model,
            base_url,
            display_name,
        }
    }

    pub fn from_config(config: &crate::config::DeepSeekConfig) -> Self {
        Self::new(
            config.api_key.clone(),
            config.model.clone(),
            config.base_url.clone(),
        )
    }
}

// ── OpenAI-compatible request/response shapes ───────────────────────────────

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

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
impl LLMProvider for DeepSeekProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn id(&self) -> &str {
        "deepseek"
    }

    fn supports_system_prompt(&self) -> bool {
        true
    }

    async fn complete(
        &self,
        system_prompt: Option<&str>,
        user_message: &str,
        config: &RequestConfig,
    ) -> Result<LLMResponse, ProviderError> {
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
            .await?;

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
            return Err(ProviderError::RateLimited { retry_after_secs: retry_after });
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
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        let config = RequestConfig {
            temperature: 0.0,
            max_tokens: 5,
        };
        self.complete(None, "ping", &config).await?;
        Ok(())
    }
}
