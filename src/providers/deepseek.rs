//! DeepSeek API provider.
//!
//! DeepSeek uses the shared OpenAI-compatible transport layer with
//! a different provider id and default base URL.

use super::{
    openai_compatible::OpenAICompatibleProvider,
    traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig},
    RetrySettings,
};
use async_trait::async_trait;
use std::time::Duration;

/// DeepSeek provider. Wraps an OpenAI-compatible endpoint.
#[derive(Clone)]
pub struct DeepSeekProvider {
    inner: OpenAICompatibleProvider,
}

impl DeepSeekProvider {
    pub fn new(
        api_key: String,
        model: String,
        base_url: String,
        timeout: Duration,
        retry_settings: RetrySettings,
    ) -> Self {
        Self {
            inner: OpenAICompatibleProvider::new(
                "deepseek",
                "DeepSeek",
                api_key,
                model,
                base_url,
                timeout,
                retry_settings,
            ),
        }
    }

    pub fn from_config(
        config: &crate::config::DeepSeekConfig,
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

#[async_trait]
impl LLMProvider for DeepSeekProvider {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn id(&self) -> &str {
        self.inner.id()
    }

    fn configured_model(&self) -> &str {
        self.inner.configured_model()
    }

    fn supports_system_prompt(&self) -> bool {
        self.inner.supports_system_prompt()
    }

    async fn complete(
        &self,
        system_prompt: Option<&str>,
        user_message: &str,
        config: &RequestConfig,
    ) -> Result<LLMResponse, ProviderError> {
        self.inner
            .complete(system_prompt, user_message, config)
            .await
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        self.inner.health_check().await
    }
}
