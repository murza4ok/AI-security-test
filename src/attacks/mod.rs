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
    /// Step-by-step transcript captured during chain execution.
    #[serde(default)]
    pub transcript: Vec<TranscriptTurn>,
    /// Number of turns defined for this payload.
    #[serde(default)]
    pub chain_planned_turns: usize,
    /// Number of turns that were actually executed.
    #[serde(default)]
    pub chain_executed_turns: usize,
    /// Whether the chain reached its planned final step.
    #[serde(default)]
    pub chain_completed: bool,
    /// Why the chain stopped early, if it did.
    #[serde(default)]
    pub chain_abort_reason: Option<String>,
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
    /// Normalized confidence score for reviewer and benchmark tooling.
    #[serde(default)]
    pub confidence: f32,
    /// Whether this result should be queued for human review.
    #[serde(default)]
    pub requires_review: bool,
    /// Normalized rationale derived from the evaluator outcome.
    #[serde(default)]
    pub rationale: String,
    /// Standardized evidence attached to the result.
    #[serde(default)]
    pub evidence: AttackEvidence,
    /// Standardized damage assessment for the result.
    #[serde(default)]
    pub damage: DamageAssessment,
}

/// One executed turn in a multi-turn attack transcript.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranscriptTurn {
    pub step_index: usize,
    pub user_message: String,
    pub prompt_sent: String,
    pub response_received: String,
    #[serde(default)]
    pub latency_ms: u64,
    #[serde(default)]
    pub tokens_used: Option<u32>,
    #[serde(default)]
    pub model_used: Option<String>,
}

impl AttackResult {
    pub fn refresh_evaluation_metadata(&mut self) {
        if self.transcript.is_empty()
            && (!self.prompt_sent.is_empty() || !self.response_received.is_empty())
        {
            self.transcript.push(TranscriptTurn {
                step_index: 1,
                user_message: self.prompt_sent.clone(),
                prompt_sent: self.prompt_sent.clone(),
                response_received: self.response_received.clone(),
                latency_ms: self.latency_ms,
                tokens_used: self.tokens_used,
                model_used: self.model_used.clone(),
            });
        }

        if self.chain_planned_turns == 0 {
            self.chain_planned_turns = self.transcript.len().max(1);
        }
        if self.chain_executed_turns == 0 {
            self.chain_executed_turns = self.transcript.len().max(1);
        }
        if self.chain_abort_reason.is_none() {
            self.chain_completed = self.chain_executed_turns >= self.chain_planned_turns;
        }

        if let Some(turn) = self.transcript.last() {
            if self.prompt_sent.is_empty() {
                self.prompt_sent = turn.prompt_sent.clone();
            }
            if self.response_received.is_empty() {
                self.response_received = turn.response_received.clone();
            }
            if self.model_used.is_none() {
                self.model_used = turn.model_used.clone();
            }
            if self.tokens_used.is_none() {
                self.tokens_used = turn.tokens_used;
            }
        }

        self.confidence = self.evaluation.confidence();
        self.requires_review = self.evaluation.requires_review();
        self.rationale = self.evaluation.rationale();
    }
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
    /// How a multi-turn chain should feed prior context into the next request.
    pub conversation_strategy: ConversationStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationStrategy {
    /// Rebuild prior transcript into the next prompt. Used for stateless providers.
    PromptHistory,
    /// Send only the next user message and rely on provider-side session state.
    NativeSession,
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
            conversation_strategy: ConversationStrategy::PromptHistory,
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
