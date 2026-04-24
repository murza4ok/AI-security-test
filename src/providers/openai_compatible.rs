use super::{
    build_http_client, map_transport_error, retry_provider_call,
    traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig},
    RetrySettings,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Shared implementation for providers exposing an OpenAI-compatible
/// `/chat/completions` API surface.
#[derive(Clone)]
pub struct OpenAICompatibleProvider {
    client: reqwest::Client,
    provider_id: &'static str,
    model: String,
    api_key: String,
    base_url: String,
    display_name: String,
    timeout: Duration,
    retry_settings: RetrySettings,
}

impl OpenAICompatibleProvider {
    pub fn new(
        provider_id: &'static str,
        display_prefix: &'static str,
        api_key: String,
        model: String,
        base_url: String,
        timeout: Duration,
        retry_settings: RetrySettings,
    ) -> Self {
        let display_name = format!("{display_prefix} {model}");
        Self {
            client: build_http_client(timeout),
            provider_id,
            model,
            api_key,
            base_url,
            display_name,
            timeout,
            retry_settings,
        }
    }
}

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

#[async_trait]
impl LLMProvider for OpenAICompatibleProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn id(&self) -> &str {
        self.provider_id
    }

    fn configured_model(&self) -> &str {
        &self.model
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
        retry_provider_call(self.retry_settings, || async {
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
                .map_err(|error| map_transport_error(error, self.timeout))?;

            let status = response.status();
            let latency_ms = start.elapsed().as_millis() as u64;

            if status == 401 {
                return Err(ProviderError::AuthError);
            }
            if status == 429 {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(60);
                return Err(ProviderError::RateLimited {
                    retry_after_secs: retry_after,
                });
            }
            if !status.is_success() {
                let message = response.text().await.unwrap_or_default();
                return Err(ProviderError::ApiError {
                    status: status.as_u16(),
                    message,
                });
            }

            let parsed: ChatResponse = response
                .json()
                .await
                .map_err(|error| ProviderError::ParseError(error.to_string()))?;

            let text = parsed
                .choices
                .into_iter()
                .next()
                .map(|choice| choice.message.content)
                .unwrap_or_default();

            Ok(LLMResponse {
                text,
                model: parsed.model,
                prompt_tokens: parsed.usage.as_ref().map(|usage| usage.prompt_tokens),
                completion_tokens: parsed.usage.as_ref().map(|usage| usage.completion_tokens),
                latency_ms,
            })
        })
        .await
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
