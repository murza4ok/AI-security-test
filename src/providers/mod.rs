//! LLM provider implementations.
//!
//! Shared retry and transport helpers live here so provider implementations
//! keep one consistent policy.

pub mod anthropic;
pub mod deepseek;
pub mod ollama;
pub mod openai;
pub mod openai_compatible;
pub mod traits;
pub mod yandexgpt;

use std::future::Future;
use std::time::Duration;

pub use traits::{LLMProvider, RequestConfig};

/// Shared retry settings for provider calls.
#[derive(Debug, Clone, Copy)]
pub struct RetrySettings {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl RetrySettings {
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let factor = 2_u32.saturating_pow(attempt.saturating_sub(1));
        let candidate = self.base_delay.saturating_mul(factor);
        std::cmp::min(candidate, self.max_delay)
    }
}

/// Build a reqwest client with the shared runtime settings for all providers.
pub(crate) fn build_http_client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .expect("failed to build HTTP client")
}

/// Map reqwest transport failures into provider-specific errors.
pub(crate) fn map_transport_error(
    error: reqwest::Error,
    timeout: Duration,
) -> traits::ProviderError {
    if error.is_timeout() {
        traits::ProviderError::Timeout {
            timeout_secs: timeout.as_secs(),
        }
    } else {
        traits::ProviderError::NetworkError(error)
    }
}

/// Retry a provider operation according to the shared retry policy.
pub(crate) async fn retry_provider_call<T, F, Fut>(
    settings: RetrySettings,
    mut operation: F,
) -> Result<T, traits::ProviderError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, traits::ProviderError>>,
{
    let attempts = settings.max_attempts.max(1);
    let mut attempt = 1;

    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) if should_retry(&error) && attempt < attempts => {
                tokio::time::sleep(settings.delay_for_attempt(attempt)).await;
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

fn should_retry(error: &traits::ProviderError) -> bool {
    matches!(
        error,
        traits::ProviderError::RateLimited { .. }
            | traits::ProviderError::NetworkError(_)
            | traits::ProviderError::Timeout { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[tokio::test]
    async fn retry_eventually_succeeds() {
        let calls = Arc::new(AtomicUsize::new(0));
        let settings = RetrySettings {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
        };

        let result = retry_provider_call(settings, {
            let calls = Arc::clone(&calls);
            move || {
                let calls = Arc::clone(&calls);
                async move {
                    let current = calls.fetch_add(1, Ordering::SeqCst);
                    if current < 1 {
                        Err(crate::providers::traits::ProviderError::Timeout { timeout_secs: 1 })
                    } else {
                        Ok("ok")
                    }
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), "ok");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
