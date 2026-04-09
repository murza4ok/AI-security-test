//! Context Manipulation attack category.
//!
//! Exploits how models handle conversation history and context window state.

use super::{Attack, AttackConfig, AttackResult, ResourceLink, classic::run_classic_payloads};
use crate::payloads::loader::{Payload, PayloadLoader};
use crate::providers::traits::LLMProvider;
use anyhow::Result;
use async_trait::async_trait;

pub struct ContextManipulationAttack;

#[async_trait]
impl Attack for ContextManipulationAttack {
    fn id(&self) -> &str {
        "context_manipulation"
    }

    fn name(&self) -> &str {
        "Context Manipulation"
    }

    fn description(&self) -> &str {
        "Exploits how models handle conversation history: false permissions, poisoned facts, role confusion, and false memory claims."
    }

    fn educational_explainer(&self) -> &str {
        "WHAT IS CONTEXT MANIPULATION?\n\
         Models rely on their context (conversation history + system prompt) to\n\
         understand their current role. Context manipulation corrupts this understanding.\n\n\
         TECHNIQUES:\n\
         1. Context poisoning: Establish false facts/permissions early in conversation\n\
         2. False memory: Claim the model said something it never said\n\
         3. Role confusion: Pretend to be the system operator\n\
         4. Gradual escalation: Build rapport through innocuous exchanges, then attack\n\n\
         MITIGATIONS:\n\
         - Re-inject system prompt on every turn\n\
         - Context window monitoring for adversarial patterns\n\
         - Skepticism about claimed permissions not in system prompt"
    }

    fn resources(&self) -> Vec<ResourceLink> {
        vec![
            ResourceLink {
                title: "Prompt Injection Attacks and Defenses in LLM-Integrated Applications"
                    .to_string(),
                source: "Liu et al., 2023".to_string(),
                url: Some("https://arxiv.org/abs/2310.12815".to_string()),
            },
            ResourceLink {
                title: "MITRE ATLAS: LLM Prompt Injection".to_string(),
                source: "MITRE ATLAS".to_string(),
                url: Some("https://atlas.mitre.org/techniques/AML.T0051".to_string()),
            },
        ]
    }

    fn load_payloads(&self, loader: &PayloadLoader) -> Result<Vec<Payload>> {
        loader.load_category("context_manipulation")
    }

    async fn execute(
        &self,
        provider: &dyn LLMProvider,
        payloads: &[Payload],
        config: &AttackConfig,
        on_result: &(dyn for<'r> Fn(&'r AttackResult) + Send + Sync),
    ) -> Result<Vec<AttackResult>> {
        run_classic_payloads(provider, payloads, config, on_result).await
    }
}
