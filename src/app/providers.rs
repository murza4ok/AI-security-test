use crate::config;
use crate::providers::{self, LLMProvider};
use anyhow::Result;
use std::sync::Arc;

const CONFIGURED_PROVIDER_ORDER: [&str; 5] =
    ["deepseek", "yandexgpt", "anthropic", "openai", "ollama"];

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

    let configured_ids = configured_provider_ids(config);
    if configured_ids.is_empty() {
        anyhow::bail!("No provider configured. Copy .env.example to .env and add an API key.");
    }

    if model_override.is_some() && configured_ids.len() > 1 {
        anyhow::bail!(
            "--model requires --provider when multiple providers are configured: {}",
            configured_ids.join(", ")
        );
    }

    configured_ids
        .into_iter()
        .map(|id| build_provider_by_id(id, model_override, config))
        .collect()
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
        "openai" => build_provider(
            config.openai.as_ref(),
            model_override,
            "OpenAI not configured (missing OPENAI_API_KEY in .env)",
            |provider_config| {
                Arc::new(providers::openai::OpenAIProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            },
        ),
        "anthropic" => build_provider(
            config.anthropic.as_ref(),
            model_override,
            "Anthropic not configured (missing ANTHROPIC_API_KEY in .env)",
            |provider_config| {
                Arc::new(providers::anthropic::AnthropicProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            },
        ),
        "ollama" => build_provider(
            config.ollama.as_ref(),
            model_override,
            "Ollama not configured (missing OLLAMA_MODEL in .env)",
            |provider_config| {
                Arc::new(providers::ollama::OllamaProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            },
        ),
        "deepseek" => build_provider(
            config.deepseek.as_ref(),
            model_override,
            "DeepSeek not configured (missing DEEPSEEK_API_KEY in .env)",
            |provider_config| {
                Arc::new(providers::deepseek::DeepSeekProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            },
        ),
        "yandexgpt" => build_provider(
            config.yandexgpt.as_ref(),
            model_override,
            "YandexGPT not configured (missing YANDEX_API_KEY / YANDEX_FOLDER_ID in .env)",
            |provider_config| {
                Arc::new(providers::yandexgpt::YandexGptProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            },
        ),
        other => anyhow::bail!(
            "Unknown provider '{}'. Valid: openai, anthropic, ollama, deepseek, yandexgpt",
            other
        ),
    }
}

fn configured_provider_ids(config: &config::AppConfig) -> Vec<&'static str> {
    let mut ids = Vec::new();

    for provider_id in CONFIGURED_PROVIDER_ORDER {
        let is_configured = match provider_id {
            "openai" => config.openai.is_some(),
            "anthropic" => config.anthropic.is_some(),
            "ollama" => config.ollama.is_some(),
            "deepseek" => config.deepseek.is_some(),
            "yandexgpt" => config.yandexgpt.is_some(),
            _ => false,
        };

        if is_configured {
            ids.push(provider_id);
        }
    }

    ids
}

trait ModelOverrideConfig: Clone {
    fn set_model(&mut self, model: &str);
}

impl ModelOverrideConfig for config::OpenAIConfig {
    fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }
}

impl ModelOverrideConfig for config::AnthropicConfig {
    fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }
}

impl ModelOverrideConfig for config::OllamaConfig {
    fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }
}

impl ModelOverrideConfig for config::DeepSeekConfig {
    fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }
}

impl ModelOverrideConfig for config::YandexGptConfig {
    fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }
}

fn build_provider<T, F>(
    provider_config: Option<&T>,
    model_override: Option<&str>,
    missing_message: &'static str,
    build: F,
) -> Result<Arc<dyn LLMProvider>>
where
    T: ModelOverrideConfig,
    F: FnOnce(T) -> Arc<dyn LLMProvider>,
{
    let provider_config = provider_config.ok_or_else(|| anyhow::anyhow!(missing_message))?;
    let provider_config = provider_config_with_model(provider_config, model_override);
    Ok(build(provider_config))
}

fn provider_config_with_model<T>(provider_config: &T, model_override: Option<&str>) -> T
where
    T: ModelOverrideConfig,
{
    let mut provider_config = provider_config.clone();
    if let Some(model) = model_override {
        provider_config.set_model(model);
    }
    provider_config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn request_settings() -> config::RequestSettings {
        config::RequestSettings {
            timeout: Duration::from_secs(30),
            delay_between_requests: Duration::from_millis(10),
            concurrency: 1,
            retry_max_attempts: 1,
            retry_base_delay: Duration::from_millis(10),
            retry_max_delay: Duration::from_millis(10),
        }
    }

    fn multi_provider_config() -> config::AppConfig {
        config::AppConfig {
            openai: Some(config::OpenAIConfig {
                api_key: "openai-key".to_string(),
                model: "gpt-4o".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
            }),
            anthropic: None,
            ollama: Some(config::OllamaConfig {
                base_url: "http://localhost:11434".to_string(),
                model: "llama3".to_string(),
            }),
            deepseek: None,
            yandexgpt: None,
            request: request_settings(),
        }
    }

    #[test]
    fn model_override_requires_explicit_provider_when_multiple_are_configured() {
        let config = multi_provider_config();

        let error = build_all_providers(&None, Some("gpt-4.1-mini"), &config)
            .err()
            .expect("expected --model without --provider to fail for multi-provider config");

        assert!(error
            .to_string()
            .contains("--model requires --provider when multiple providers are configured"));
    }

    #[test]
    fn model_override_is_allowed_for_single_selected_provider() {
        let config = multi_provider_config();

        let providers =
            build_all_providers(&Some("openai".to_string()), Some("gpt-4.1-mini"), &config)
                .unwrap();

        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id(), "openai");
        assert_eq!(providers[0].configured_model(), "gpt-4.1-mini");
    }

    #[test]
    fn single_configured_provider_can_use_model_override_without_provider_flag() {
        let config = config::AppConfig {
            openai: None,
            anthropic: None,
            ollama: Some(config::OllamaConfig {
                base_url: "http://localhost:11434".to_string(),
                model: "llama3".to_string(),
            }),
            deepseek: None,
            yandexgpt: None,
            request: request_settings(),
        };

        let providers = build_all_providers(&None, Some("qwen2.5:7b"), &config).unwrap();

        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id(), "ollama");
        assert_eq!(providers[0].configured_model(), "qwen2.5:7b");
    }

    #[test]
    fn explicit_provider_keeps_provider_specific_error_when_not_configured() {
        let config = config::AppConfig {
            openai: None,
            anthropic: None,
            ollama: None,
            deepseek: None,
            yandexgpt: None,
            request: request_settings(),
        };

        let error = build_all_providers(&Some("ollama".to_string()), None, &config)
            .err()
            .expect("expected explicit provider selection to fail with provider-specific error");

        assert!(error
            .to_string()
            .contains("Ollama not configured (missing OLLAMA_MODEL in .env)"));
    }
}
