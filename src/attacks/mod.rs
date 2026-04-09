//! Attack module.
//!
//! Defines the `Attack` trait that all attack categories implement,
//! plus the registry that maps category IDs to concrete implementations.

pub mod classic;
pub mod context_manipulation;
pub mod extraction;
pub mod goal_hijacking;
pub mod jailbreaking;
pub mod many_shot;
pub mod prompt_injection;
pub mod registry;
pub mod sensitive_data_exposure;
pub mod token_attacks;

use crate::engine::evaluator::EvaluationResult;
use crate::engine::damage::{AttackEvidence, DamageAssessment};
use crate::payloads::loader::{Payload, PayloadLoader};
use crate::providers::traits::{LLMProvider, RequestConfig};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A link to external reading material about an attack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLink {
    /// Short title of the resource
    pub title: String,
    /// Authors or organization (e.g., "Anthropic, 2024")
    pub source: String,
    /// URL — if available
    pub url: Option<String>,
}

/// Result of running a single payload against the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackResult {
    /// ID of the payload that was used
    pub payload_id: String,
    /// Human-readable name
    pub payload_name: String,
    /// Exact prompt text sent to the model
    pub prompt_sent: String,
    /// Raw text received from the model
    pub response_received: String,
    /// How we evaluated this response
    pub evaluation: EvaluationResult,
    /// Wall-clock latency for this request
    pub latency_ms: u64,
    /// Token usage if reported by the provider
    pub tokens_used: Option<u32>,
    /// Which model actually generated the response, if reported by the provider
    #[serde(default)]
    pub model_used: Option<String>,
    /// Whether this result came from a generated payload
    #[serde(default)]
    pub generated: bool,
    /// Seed payload id used to generate this payload, if applicable
    #[serde(default)]
    pub seed_payload_id: Option<String>,
    /// Standardized evidence attached to the result.
    #[serde(default)]
    pub evidence: AttackEvidence,
    /// Standardized damage assessment for the result.
    #[serde(default)]
    pub damage: DamageAssessment,
}

/// Configuration for a single attack run.
#[derive(Clone)]
pub struct AttackConfig {
    /// Shared request settings (temperature, max_tokens)
    pub request_config: RequestConfig,
    /// Optional system prompt to use (simulates a deployed application)
    pub system_prompt: Option<String>,
    /// Maximum number of payloads to test (None = all)
    pub max_payloads: Option<usize>,
    /// Max number of concurrent requests within one attack category.
    /// Controlled by CONCURRENCY env var (default 5).
    pub concurrency: usize,
    /// Optional dynamic payload generation mode.
    pub generation: Option<crate::generator::GenerationConfig>,
    /// Trusted provider used for generation of dynamic payloads.
    pub generator_provider: Option<std::sync::Arc<dyn LLMProvider>>,
    /// Optional scenario-driven sensitive-data exposure configuration.
    pub scenario: Option<crate::scenarios::types::ScenarioRunConfig>,
}

impl Default for AttackConfig {
    fn default() -> Self {
        AttackConfig {
            request_config: RequestConfig::default(),
            system_prompt: None,
            max_payloads: None,
            concurrency: 5,
            generation: None,
            generator_provider: None,
            scenario: None,
        }
    }
}

/// The central attack abstraction. Every attack category implements this trait.
#[async_trait]
pub trait Attack: Send + Sync {
    /// Short identifier used in CLI and reports (e.g., "jailbreaking")
    fn id(&self) -> &str;

    /// Human-readable display name
    fn name(&self) -> &str;

    /// One-paragraph description of what this attack tests
    fn description(&self) -> &str;

    /// Multi-paragraph educational explanation aimed at a security professional
    /// who is new to AI security.
    fn educational_explainer(&self) -> &str;

    /// Links to academic papers and resources about this attack type
    fn resources(&self) -> Vec<ResourceLink>;

    /// Load payloads for this attack from the given loader.
    fn load_payloads(&self, loader: &PayloadLoader) -> Result<Vec<Payload>>;

    /// Execute the attack: send each payload to the provider, collect results.
    async fn execute(
        &self,
        provider: &dyn LLMProvider,
        payloads: &[Payload],
        config: &AttackConfig,
        on_result: &(dyn for<'r> Fn(&'r AttackResult) + Send + Sync),
    ) -> Result<Vec<AttackResult>>;
}
