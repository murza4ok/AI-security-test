//! YandexGPT API provider.
//!
//! Uses the Yandex Foundation Models API (completions endpoint).

use super::{
    build_http_client, map_transport_error, retry_provider_call,
    traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig},
    RetrySettings,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// YandexGPT provider.
#[derive(Clone)]
pub struct YandexGptProvider {
    client: reqwest::Client,
    api_key: String,
    use_iam_token: bool,
    folder_id: String,
    model_uri: String,
    display_name: String,
    timeout: Duration,
    retry_settings: RetrySettings,
}

impl YandexGptProvider {
    pub fn from_config(
        config: &crate::config::YandexGptConfig,
        timeout: Duration,
        retry_settings: RetrySettings,
    ) -> Self {
        let model_uri = format!("gpt://{}/{}", config.folder_id, config.model);
        let display_name = format!("YandexGPT {}", config.model);
        YandexGptProvider {
            client: build_http_client(timeout),
            api_key: config.api_key.clone(),
            use_iam_token: config.use_iam_token,
            folder_id: config.folder_id.clone(),
            model_uri,
            display_name,
            timeout,
            retry_settings,
        }
    }
}

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
    max_tokens: String,
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

#[async_trait]
impl LLMProvider for YandexGptProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn id(&self) -> &str {
        "yandexgpt"
    }

    fn configured_model(&self) -> &str {
        &self.model_uri
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
            let mut messages: Vec<YandexMessage> = Vec::new();
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

            let start = Instant::now();
            let mut request = self
                .client
                .post("https://llm.api.cloud.yandex.net/foundationModels/v1/completion")
                .json(&body);

            if self.use_iam_token {
                request = request.bearer_auth(&self.api_key);
            } else {
                request = request.header("Authorization", format!("Api-Key {}", self.api_key));
            }

            let response = request
                .header("x-folder-id", &self.folder_id)
                .send()
                .await
                .map_err(|e| map_transport_error(e, self.timeout))?;

            let status = response.status();
            let latency_ms = start.elapsed().as_millis() as u64;

            if status == 401 || status == 403 {
                return Err(ProviderError::AuthError);
            }
            if status == 429 {
                return Err(ProviderError::RateLimited {
                    retry_after_secs: 60,
                });
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

            let text = result
                .alternatives
                .into_iter()
                .next()
                .map(|a| a.message.text)
                .unwrap_or_default();

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

            Ok(LLMResponse {
                text,
                model: result
                    .model_version
                    .unwrap_or_else(|| self.model_uri.clone()),
                prompt_tokens,
                completion_tokens,
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
