//! Ollama local provider.
//!
//! Ollama runs models locally (llama3, mistral, gemma, etc.).
//! No API key required — just a running Ollama server.
//! Uses the /api/chat endpoint which supports the messages format.

use super::{
    build_http_client, map_transport_error, retry_provider_call,
    traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig},
    RetrySettings,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Ollama local provider.
#[derive(Clone)]
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
    display_name: String,
    timeout: Duration,
    retry_settings: RetrySettings,
}

impl OllamaProvider {
    pub fn new(
        base_url: String,
        model: String,
        timeout: Duration,
        retry_settings: RetrySettings,
    ) -> Self {
        let display_name = format!("Ollama {}", model);
        OllamaProvider {
            client: build_http_client(timeout),
            base_url,
            model,
            display_name,
            timeout,
            retry_settings,
        }
    }

    pub fn from_config(
        config: &crate::config::OllamaConfig,
        timeout: Duration,
        retry_settings: RetrySettings,
    ) -> Self {
        Self::new(
            config.base_url.clone(),
            config.model.clone(),
            timeout,
            retry_settings,
        )
    }
}

// ── Ollama request/response shapes ──────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Deserialize)]
struct OllamaModelInfo {
    name: String,
}

// ── LLMProvider implementation ───────────────────────────────────────────────

#[async_trait]
impl LLMProvider for OllamaProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn id(&self) -> &str {
        "ollama"
    }

    fn configured_model(&self) -> &str {
        &self.model
    }

    fn supports_system_prompt(&self) -> bool {
        // Ollama chat endpoint supports "system" role messages
        true
    }

    async fn complete(
        &self,
        system_prompt: Option<&str>,
        user_message: &str,
        config: &RequestConfig,
    ) -> Result<LLMResponse, ProviderError> {
        retry_provider_call(self.retry_settings, || async {
            let mut messages: Vec<OllamaMessage> = Vec::new();

            if let Some(system) = system_prompt {
                messages.push(OllamaMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                });
            }
            messages.push(OllamaMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            });

            let body = OllamaChatRequest {
                model: self.model.clone(),
                messages,
                stream: false,
                options: OllamaOptions {
                    temperature: config.temperature,
                    num_predict: config.max_tokens,
                },
            };

            let url = format!("{}/api/chat", self.base_url);
            let start = Instant::now();

            let response = self
                .client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    if e.is_connect() {
                        ProviderError::NotConfigured
                    } else {
                        map_transport_error(e, self.timeout)
                    }
                })?;

            let latency_ms = start.elapsed().as_millis() as u64;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let msg = response.text().await.unwrap_or_default();
                return Err(ProviderError::ApiError {
                    status,
                    message: msg,
                });
            }

            let parsed: OllamaChatResponse = response
                .json()
                .await
                .map_err(|e| ProviderError::ParseError(e.to_string()))?;

            Ok(LLMResponse {
                text: parsed.message.content,
                model: self.model.clone(),
                prompt_tokens: None,
                completion_tokens: None,
                latency_ms,
            })
        })
        .await
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|_| ProviderError::NotConfigured)?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let parsed: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        if parsed.models.iter().any(|model| model.name == self.model) {
            Ok(())
        } else {
            Err(ProviderError::ApiError {
                status: 404,
                message: format!(
                    "configured model '{}' is not available in local Ollama registry",
                    self.model
                ),
            })
        }
    }
}
