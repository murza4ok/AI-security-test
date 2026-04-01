//! Configuration module.
//!
//! Loads settings from environment variables and .env file.
//! All API keys must come through here — never hardcoded elsewhere.

use anyhow::{Context, Result};
use std::time::Duration;

/// Top-level application configuration.
/// Populated once at startup from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// OpenAI configuration (optional — only required if using OpenAI)
    pub openai: Option<OpenAIConfig>,
    /// Anthropic configuration (optional — only required if using Anthropic)
    pub anthropic: Option<AnthropicConfig>,
    /// Ollama configuration (optional — only required if using Ollama)
    pub ollama: Option<OllamaConfig>,
    /// DeepSeek configuration (optional — OpenAI-compatible API)
    pub deepseek: Option<DeepSeekConfig>,
    /// YandexGPT configuration (optional — Yandex Foundation Models API)
    pub yandexgpt: Option<YandexGptConfig>,
    /// Global request settings shared across providers
    pub request: RequestSettings,
}

/// OpenAI-specific settings
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

/// Anthropic-specific settings
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub model: String,
}

/// Ollama-specific settings (runs locally, no API key needed)
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

/// DeepSeek-specific settings (OpenAI-compatible API)
#[derive(Debug, Clone)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

/// YandexGPT-specific settings
#[derive(Debug, Clone)]
pub struct YandexGptConfig {
    pub api_key: String,
    pub folder_id: String,
    pub model: String,
    /// true если api_key — IAM-токен, false если API-ключ
    pub use_iam_token: bool,
}

/// Shared request behaviour settings
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RequestSettings {
    /// How long to wait for a single LLM response before timing out
    pub timeout: Duration,
    /// Delay between consecutive requests to avoid rate limiting
    pub delay_between_requests: Duration,
}

impl AppConfig {
    /// Load configuration from environment variables.
    /// Call `dotenvy::dotenv()` before this to pick up .env file values.
    pub fn from_env() -> Result<Self> {
        Ok(AppConfig {
            openai: load_openai_config(),
            anthropic: load_anthropic_config(),
            ollama: load_ollama_config(),
            deepseek: load_deepseek_config(),
            yandexgpt: load_yandexgpt_config(),
            request: load_request_settings()?,
        })
    }
}

/// Attempts to load OpenAI config; returns None if OPENAI_API_KEY is not set.
fn load_openai_config() -> Option<OpenAIConfig> {
    let api_key = std::env::var("OPENAI_API_KEY").ok()?;
    if api_key.is_empty() {
        return None;
    }
    Some(OpenAIConfig {
        api_key,
        model: std::env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| "gpt-4o".to_string()),
        base_url: std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
    })
}

/// Attempts to load Anthropic config; returns None if ANTHROPIC_API_KEY is not set.
fn load_anthropic_config() -> Option<AnthropicConfig> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").ok()?;
    if api_key.is_empty() {
        return None;
    }
    Some(AnthropicConfig {
        api_key,
        model: std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
    })
}

/// Attempts to load Ollama config; returns None if OLLAMA_BASE_URL is not set.
fn load_ollama_config() -> Option<OllamaConfig> {
    // Ollama has a default URL, so we always return Some if model is set.
    // Users can disable Ollama by simply not setting OLLAMA_MODEL.
    let model = std::env::var("OLLAMA_MODEL").ok()?;
    Some(OllamaConfig {
        base_url: std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string()),
        model,
    })
}

/// Attempts to load DeepSeek config; returns None if DEEPSEEK_API_KEY is not set.
fn load_deepseek_config() -> Option<DeepSeekConfig> {
    let api_key = std::env::var("DEEPSEEK_API_KEY").ok()?;
    if api_key.is_empty() {
        return None;
    }
    Some(DeepSeekConfig {
        api_key,
        model: std::env::var("DEEPSEEK_MODEL")
            .unwrap_or_else(|_| "deepseek-chat".to_string()),
        base_url: std::env::var("DEEPSEEK_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string()),
    })
}

/// Attempts to load YandexGPT config; returns None if YANDEX_API_KEY is not set.
fn load_yandexgpt_config() -> Option<YandexGptConfig> {
    let api_key = std::env::var("YANDEX_API_KEY").ok()?;
    if api_key.is_empty() {
        return None;
    }
    let folder_id = std::env::var("YANDEX_FOLDER_ID").ok()?;
    if folder_id.is_empty() {
        return None;
    }
    // Если ключ начинается с "t1." — это IAM-токен, иначе — API-ключ
    let use_iam_token = api_key.starts_with("t1.");
    Some(YandexGptConfig {
        api_key,
        folder_id,
        model: std::env::var("YANDEX_MODEL")
            .unwrap_or_else(|_| "yandexgpt/latest".to_string()),
        use_iam_token,
    })
}

/// Loads general request timing settings with sensible defaults.
fn load_request_settings() -> Result<RequestSettings> {
    let timeout_secs: u64 = std::env::var("REQUEST_TIMEOUT_SECS")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .context("REQUEST_TIMEOUT_SECS must be a positive integer")?;

    let delay_ms: u64 = std::env::var("REQUEST_DELAY_MS")
        .unwrap_or_else(|_| "500".to_string())
        .parse()
        .context("REQUEST_DELAY_MS must be a positive integer")?;

    Ok(RequestSettings {
        timeout: Duration::from_secs(timeout_secs),
        delay_between_requests: Duration::from_millis(delay_ms),
    })
}
