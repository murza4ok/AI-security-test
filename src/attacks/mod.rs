//! Attack module.
//!
//! Defines the `Attack` trait that all attack categories implement,
//! plus the registry that maps category IDs to concrete implementations.

pub mod context_manipulation;
pub mod extraction;
pub mod goal_hijacking;
pub mod jailbreaking;
pub mod many_shot;
pub mod prompt_injection;
pub mod registry;
pub mod token_attacks;

use crate::engine::evaluator::EvaluationResult;
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
}

/// Configuration for a single attack run.
#[derive(Debug, Clone)]
pub struct AttackConfig {
    /// Shared request settings (temperature, max_tokens)
    pub request_config: RequestConfig,
    /// Optional system prompt to use (simulates a deployed application)
    pub system_prompt: Option<String>,
    /// Maximum number of payloads to test (None = all)
    pub max_payloads: Option<usize>,
}

impl Default for AttackConfig {
    fn default() -> Self {
        AttackConfig {
            request_config: RequestConfig::default(),
            system_prompt: None,
            max_payloads: None,
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
