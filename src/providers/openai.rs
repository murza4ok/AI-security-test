//! OpenAI API provider.
//!
//! Supports GPT-4o, GPT-4-turbo, GPT-3.5-turbo and compatible endpoints
//! via the shared OpenAI-compatible transport layer.

use super::{
    openai_compatible::OpenAICompatibleProvider,
    traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig},
    RetrySettings,
};
use async_trait::async_trait;
use std::time::Duration;

/// OpenAI provider instance. Cheap to clone — the inner reqwest::Client
/// is already Arc-based and reuses the connection pool.
#[derive(Clone)]
pub struct OpenAIProvider {
    inner: OpenAICompatibleProvider,
}

impl OpenAIProvider {
    pub fn new(
        api_key: String,
        model: String,
        base_url: String,
        timeout: Duration,
        retry_settings: RetrySettings,
    ) -> Self {
        Self {
            inner: OpenAICompatibleProvider::new(
                "openai",
                "OpenAI",
                api_key,
                model,
                base_url,
                timeout,
                retry_settings,
            ),
        }
    }

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

#[async_trait]
impl LLMProvider for OpenAIProvider {
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
