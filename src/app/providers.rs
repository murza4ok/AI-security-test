use crate::config;
use crate::providers::{self, LLMProvider};
use anyhow::Result;
use std::sync::Arc;

pub fn build_generation_provider(
    config: &config::AppConfig,
    model_override: Option<&str>,
) -> Result<Arc<dyn LLMProvider>> {
    build_provider_by_id("deepseek", model_override, config)
}

pub fn build_all_providers(
    override_id: &Option<String>,
    model_override: Option<&str>,
    config: &config::AppConfig,
) -> Result<Vec<Arc<dyn LLMProvider>>> {
    if let Some(id) = override_id.as_deref() {
        return build_provider_by_id(id, model_override, config).map(|provider| vec![provider]);
    }

    let mut list: Vec<Arc<dyn LLMProvider>> = Vec::new();
    let timeout = config.request.timeout;
    let retry_settings = providers::RetrySettings {
        max_attempts: config.request.retry_max_attempts,
        base_delay: config.request.retry_base_delay,
        max_delay: config.request.retry_max_delay,
    };

    if let Some(provider_config) = &config.deepseek {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(providers::deepseek::DeepSeekProvider::from_config(
            &provider_config,
            timeout,
            retry_settings,
        )));
    }
    if let Some(provider_config) = &config.yandexgpt {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(
            providers::yandexgpt::YandexGptProvider::from_config(
                &provider_config,
                timeout,
                retry_settings,
            ),
        ));
    }
    if let Some(provider_config) = &config.anthropic {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(
            providers::anthropic::AnthropicProvider::from_config(
                &provider_config,
                timeout,
                retry_settings,
            ),
        ));
    }
    if let Some(provider_config) = &config.openai {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(providers::openai::OpenAIProvider::from_config(
            &provider_config,
            timeout,
            retry_settings,
        )));
    }
    if let Some(provider_config) = &config.ollama {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(providers::ollama::OllamaProvider::from_config(
            &provider_config,
            timeout,
            retry_settings,
        )));
    }

    if list.is_empty() {
        anyhow::bail!("No provider configured. Copy .env.example to .env and add an API key.");
    }

    Ok(list)
}

pub fn build_provider_by_id(
    id: &str,
    model_override: Option<&str>,
    config: &config::AppConfig,
) -> Result<Arc<dyn LLMProvider>> {
    let timeout = config.request.timeout;
    let retry_settings = providers::RetrySettings {
        max_attempts: config.request.retry_max_attempts,
        base_delay: config.request.retry_base_delay,
        max_delay: config.request.retry_max_delay,
    };

    match id {
        "openai" => config
            .openai
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::openai::OpenAIProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            })
            .ok_or_else(|| anyhow::anyhow!("OpenAI not configured (missing OPENAI_API_KEY in .env)")),
        "anthropic" => config
            .anthropic
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::anthropic::AnthropicProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Anthropic not configured (missing ANTHROPIC_API_KEY in .env)")
            }),
        "ollama" => config
            .ollama
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::ollama::OllamaProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            })
            .ok_or_else(|| anyhow::anyhow!("Ollama not configured (missing OLLAMA_MODEL in .env)")),
        "deepseek" => config
            .deepseek
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::deepseek::DeepSeekProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            })
            .ok_or_else(|| anyhow::anyhow!("DeepSeek not configured (missing DEEPSEEK_API_KEY in .env)")),
        "yandexgpt" => config
            .yandexgpt
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::yandexgpt::YandexGptProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "YandexGPT not configured (missing YANDEX_API_KEY / YANDEX_FOLDER_ID in .env)"
                )
            }),
        other => anyhow::bail!(
            "Unknown provider '{}'. Valid: openai, anthropic, ollama, deepseek, yandexgpt",
            other
        ),
    }
}
