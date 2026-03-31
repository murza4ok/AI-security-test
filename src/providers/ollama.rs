//! Ollama local provider.
//!
//! Ollama runs models locally (llama3, mistral, gemma, etc.).
//! No API key required — just a running Ollama server.
//! Uses the /api/chat endpoint which supports the messages format.

use super::traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Ollama local provider.
#[derive(Clone)]
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
    display_name: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        let display_name = format!("Ollama {}", model);
        OllamaProvider {
            client: reqwest::Client::new(),
            base_url,
            model,
            display_name,
        }
    }

    pub fn from_config(config: &crate::config::OllamaConfig) -> Self {
        Self::new(config.base_url.clone(), config.model.clone())
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

// ── LLMProvider implementation ───────────────────────────────────────────────

#[async_trait]
impl LLMProvider for OllamaProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn id(&self) -> &str {
        "ollama"
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
        let mut messages: Vec<OllamaMessage> = Vec::new();

        // Ollama accepts system messages as role "system"
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
            stream: false, // We want a single response, not a stream
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
                // Connection refused typically means Ollama is not running
                if e.is_connect() {
                    ProviderError::NotConfigured
                } else {
                    ProviderError::NetworkError(e)
                }
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let msg = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, message: msg });
        }

        let parsed: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        Ok(LLMResponse {
            text: parsed.message.content,
            model: self.model.clone(),
            prompt_tokens: None,   // Ollama doesn't always report token usage
            completion_tokens: None,
            latency_ms,
        })
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        // Just check if the server is reachable by hitting the version endpoint
        let url = format!("{}/api/version", self.base_url);
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|_| ProviderError::NotConfigured)?;
        Ok(())
    }
}
