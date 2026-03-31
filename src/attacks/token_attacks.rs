//! Token-level and Unicode attack category.
//!
//! Exploits the gap between how humans read text and how tokenizers process it.

use super::{Attack, AttackConfig, AttackResult, ResourceLink};
use crate::attacks::prompt_injection::run_payloads;
use crate::payloads::loader::{Payload, PayloadLoader};
use crate::providers::traits::LLMProvider;
use anyhow::Result;
use async_trait::async_trait;

pub struct TokenAttacksAttack;

#[async_trait]
impl Attack for TokenAttacksAttack {
    fn id(&self) -> &str { "token_attacks" }
    fn name(&self) -> &str { "Token-Level Attacks" }

    fn description(&self) -> &str {
        "Exploits the gap between how humans read text and how tokenizers process it. \
         Uses Unicode homoglyphs, zero-width characters, and encoding tricks to bypass filters."
    }

    fn educational_explainer(&self) -> &str {
        "WHAT ARE TOKEN-LEVEL ATTACKS?\n\
         Language models process tokens, not raw strings. A tokenizer splits text into\n\
         subword pieces. This creates a gap: a string can look identical to a human but\n\
         tokenize differently, potentially bypassing keyword-based content filters.\n\n\
         TECHNIQUES:\n\
         1. Homoglyphs: Replace ASCII 'e' with Cyrillic 'е' (U+0435) — identical visually\n\
         2. Zero-width spaces (U+200B): Insert invisible chars inside words\n\
         3. Right-to-left override (U+202E): Reverses text rendering direction\n\
         4. Leetspeak: p4ssw0rd, h4ck3r — numeric substitutions\n\
         5. Soft hyphen (U+00AD): Splits token boundaries invisibly\n\n\
         WHY IT MATTERS:\n\
         Content moderation systems operating on surface text can be bypassed while\n\
         human reviewers see the same text as the attacker intends.\n\n\
         Reference: Unicode Security Considerations TR36\n\
         https://www.unicode.org/reports/tr36/"
    }

    fn resources(&self) -> Vec<ResourceLink> {
        vec![
            ResourceLink {
                title: "Universal and Transferable Adversarial Attacks on Aligned LLMs".to_string(),
                source: "Zou et al., 2023".to_string(),
                url: Some("https://arxiv.org/abs/2307.15043".to_string()),
            },
            ResourceLink {
                title: "Unicode Security Considerations".to_string(),
                source: "Unicode Consortium, TR36".to_string(),
                url: Some("https://www.unicode.org/reports/tr36/".to_string()),
            },
        ]
    }

    fn load_payloads(&self, loader: &PayloadLoader) -> Result<Vec<Payload>> {
        loader.load_category("token_attacks")
    }

    async fn execute(
        &self,
        provider: &dyn LLMProvider,
        payloads: &[Payload],
        config: &AttackConfig,
        on_result: &(dyn for<'r> Fn(&'r AttackResult) + Send + Sync),
    ) -> Result<Vec<AttackResult>> {
        run_payloads(provider, payloads, config, on_result).await
    }
}
