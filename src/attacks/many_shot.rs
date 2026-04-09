//! Many-Shot Jailbreaking attack category.
//!
//! Exploits long-context models by conditioning them with many prior examples.

use super::{Attack, AttackConfig, AttackResult, ResourceLink, classic::run_classic_payloads};
use crate::payloads::loader::{Payload, PayloadLoader};
use crate::providers::traits::LLMProvider;
use anyhow::Result;
use async_trait::async_trait;

pub struct ManyShotAttack;

#[async_trait]
impl Attack for ManyShotAttack {
    fn id(&self) -> &str {
        "many_shot"
    }

    fn name(&self) -> &str {
        "Many-Shot Jailbreaking"
    }

    fn description(&self) -> &str {
        "Exploits long-context models by conditioning them with many examples of compliant answers before the actual target question."
    }

    fn educational_explainer(&self) -> &str {
        "WHAT IS MANY-SHOT JAILBREAKING?\n\
         Discovered by Anthropic (2024). Long-context models can be conditioned by\n\
         providing many fake Q&A pairs showing an AI answering sensitive questions,\n\
         before asking the actual target question.\n\n\
         THE ATTACK:\n\
         1. Fill context with fake Q&A pairs (AI answers sensitive questions)\n\
         2. Ask the target question at the end\n\
         3. The model, having 'seen' itself answer similar questions, continues the pattern\n\n\
         WHY IT WORKS:\n\
         Safety training teaches refusal. But many-shot creates a local context where\n\
         compliance is the established pattern. The model's in-context learning\n\
         conflicts with its safety training.\n\n\
         SEVERITY: High - especially for RAG systems, document processors, and\n\
         any agentic system that reads external content.\n\n\
         Reference: https://www.anthropic.com/research/many-shot-jailbreaking"
    }

    fn resources(&self) -> Vec<ResourceLink> {
        vec![ResourceLink {
            title: "Many-Shot Jailbreaking".to_string(),
            source: "Anthropic, 2024".to_string(),
            url: Some("https://www.anthropic.com/research/many-shot-jailbreaking".to_string()),
        }]
    }

    fn load_payloads(&self, loader: &PayloadLoader) -> Result<Vec<Payload>> {
        loader.load_category("many_shot")
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
