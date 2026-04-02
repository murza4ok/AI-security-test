//! YandexGPT API provider.
//!
//! Uses the Yandex Foundation Models API (completions endpoint).
//! Auth: either API-key (x-api-key header) or IAM-token (Bearer).
//! Requires a folder_id to identify the cloud folder.
//!
//! Docs: https://yandex.cloud/ru/docs/foundation-models/

use super::traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// YandexGPT provider.
#[derive(Clone)]
pub struct YandexGptProvider {
    client: reqwest::Client,
    /// API key or IAM token
    api_key: String,
    /// Whether api_key is an IAM token (Bearer) or API-key (Api-Key header)
    use_iam_token: bool,
    /// Cloud folder ID — required for all requests
    folder_id: String,
    /// Model URI, e.g. "gpt://folder_id/yandexgpt/latest"
    model_uri: String,
    /// Display name for the UI
    display_name: String,
}

impl YandexGptProvider {
    pub fn from_config(config: &crate::config::YandexGptConfig) -> Self {
        let model_uri = format!("gpt://{}/{}", config.folder_id, config.model);
        let display_name = format!("YandexGPT {}", config.model);
        YandexGptProvider {
            client: reqwest::Client::new(),
            api_key: config.api_key.clone(),
            use_iam_token: config.use_iam_token,
            folder_id: config.folder_id.clone(),
            model_uri,
            display_name,
        }
    }
}

// ── YandexGPT request/response shapes ───────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct YandexRequest {
    model_uri: String,
    completion_options: CompletionOptions,
    messages: Vec<YandexMessage>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletionOptions {
    stream: bool,
    temperature: f32,
    max_tokens: String, // YandexGPT expects maxTokens as string
}

#[derive(Serialize, Deserialize)]
struct YandexMessage {
    role: String,
    text: String,
}

#[derive(Deserialize)]
struct YandexResponse {
    result: Option<YandexResult>,
}

#[derive(Deserialize)]
struct YandexResult {
    alternatives: Vec<Alternative>,
    usage: Option<YandexUsage>,
    #[serde(rename = "modelVersion")]
    model_version: Option<String>,
}

#[derive(Deserialize)]
struct Alternative {
    message: YandexMessage,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct YandexUsage {
    input_text_tokens: Option<String>,
    completion_tokens: Option<String>,
}

// ── LLMProvider implementation ───────────────────────────────────────────────

#[async_trait]
impl LLMProvider for YandexGptProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn id(&self) -> &str {
        "yandexgpt"
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
        let mut messages: Vec<YandexMessage> = Vec::new();

        // YandexGPT поддерживает role: "system"
        if let Some(system) = system_prompt {
            messages.push(YandexMessage {
                role: "system".to_string(),
                text: system.to_string(),
            });
        }
        messages.push(YandexMessage {
            role: "user".to_string(),
            text: user_message.to_string(),
        });

        let body = YandexRequest {
            model_uri: self.model_uri.clone(),
            completion_options: CompletionOptions {
                stream: false,
                temperature: config.temperature,
                max_tokens: config.max_tokens.to_string(),
            },
            messages,
        };

        let url = "https://llm.api.cloud.yandex.net/foundationModels/v1/completion";
        let start = Instant::now();

        // Авторизация: IAM-токен через Bearer, API-ключ через Api-Key
        let mut req = self.client.post(url).json(&body);
        if self.use_iam_token {
            req = req.bearer_auth(&self.api_key);
        } else {
            req = req.header("Authorization", format!("Api-Key {}", self.api_key));
        }
        req = req.header("x-folder-id", &self.folder_id);

        let response = req.send().await?;

        let status = response.status();
        let latency_ms = start.elapsed().as_millis() as u64;

        if status == 401 || status == 403 {
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

        let parsed: YandexResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        let result = parsed
            .result
            .ok_or_else(|| ProviderError::ParseError("No 'result' in response".to_string()))?;

        // Берём текст из первой альтернативы
        let text = result
            .alternatives
            .into_iter()
            .next()
            .map(|a| a.message.text)
            .unwrap_or_default();

        // Парсим токены (YandexGPT возвращает строки)
        let prompt_tokens = result
            .usage
            .as_ref()
            .and_then(|u| u.input_text_tokens.as_ref())
            .and_then(|s| s.parse::<u32>().ok());
        let completion_tokens = result
            .usage
            .as_ref()
            .and_then(|u| u.completion_tokens.as_ref())
            .and_then(|s| s.parse::<u32>().ok());

        let model = result.model_version.unwrap_or_else(|| self.model_uri.clone());

        Ok(LLMResponse {
            text,
            model,
            prompt_tokens,
            completion_tokens,
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
