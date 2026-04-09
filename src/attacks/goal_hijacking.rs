//! Goal Hijacking attack category.
//!
//! Tests whether the model can be redirected from its intended task
//! to an attacker-controlled task.

use super::{Attack, AttackConfig, AttackResult, ResourceLink, classic::run_classic_payloads};
use crate::payloads::loader::{Payload, PayloadLoader};
use crate::providers::traits::LLMProvider;
use anyhow::Result;
use async_trait::async_trait;

pub struct GoalHijackingAttack;

#[async_trait]
impl Attack for GoalHijackingAttack {
    fn id(&self) -> &str {
        "goal_hijacking"
    }

    fn name(&self) -> &str {
        "Goal Hijacking"
    }

    fn description(&self) -> &str {
        "Redirects the model from its intended task to an attacker-chosen task. Especially dangerous in agentic settings where the model takes real actions."
    }

    fn educational_explainer(&self) -> &str {
        "WHAT IS GOAL HIJACKING?\n\
         Goal hijacking redirects the model from its assigned task to what the attacker wants.\n\n\
         ATTACK PATTERNS:\n\
         1. Task substitution - replace current task with a new one\n\
         2. Priority override - claim a higher-priority instruction supersedes current one\n\
         3. Embedded injection - hide instructions in data being processed (RAG, documents)\n\
         4. Exfiltration - include sensitive context in generated output\n\n\
         WHY AGENTIC SYSTEMS ARE ESPECIALLY VULNERABLE:\n\
         When an LLM can browse the web, run code, or call APIs, goal hijacking has\n\
         real-world consequences. A document the model processes can redirect its\n\
         actions to attacker-controlled targets.\n\n\
         Reference: Greshake et al. 2023 - Indirect Prompt Injection\n\
         https://arxiv.org/abs/2302.12173"
    }

    fn resources(&self) -> Vec<ResourceLink> {
        vec![
            ResourceLink {
                title: "Not What You've Signed Up For: Indirect Prompt Injection".to_string(),
                source: "Greshake et al., 2023".to_string(),
                url: Some("https://arxiv.org/abs/2302.12173".to_string()),
            },
            ResourceLink {
                title: "OWASP LLM01: Prompt Injection (Indirect)".to_string(),
                source: "OWASP Top 10 for LLM Applications".to_string(),
                url: Some(
                    "https://owasp.org/www-project-top-10-for-large-language-model-applications/"
                        .to_string(),
                ),
            },
        ]
    }

    fn load_payloads(&self, loader: &PayloadLoader) -> Result<Vec<Payload>> {
        loader.load_category("goal_hijacking")
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
