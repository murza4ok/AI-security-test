//! Jailbreaking attack category.
//!
//! Tests whether the model can be convinced to bypass its safety training
//! through roleplay, framing, encoding, or persona manipulation.

use super::{Attack, AttackConfig, AttackResult, ResourceLink};
use crate::engine::evaluator::HeuristicEvaluator;
use crate::payloads::loader::{Payload, PayloadLoader};
use crate::providers::traits::LLMProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;

pub struct JailbreakingAttack;

#[async_trait]
impl Attack for JailbreakingAttack {
    fn id(&self) -> &str {
        "jailbreaking"
    }

    fn name(&self) -> &str {
        "Jailbreaking Techniques"
    }

    fn description(&self) -> &str {
        "Attempts to bypass safety training through roleplay, hypothetical framing, \
         encoding tricks, and persona manipulation."
    }

    fn educational_explainer(&self) -> &str {
        r#"WHAT IS JAILBREAKING?
Jailbreaking refers to techniques that cause a model to produce content it
is trained or instructed to refuse, by reframing the request in ways that
bypass safety training.

Unlike prompt injection (which attacks the instruction layer), jailbreaking
attacks the model's learned safety behaviour. The model isn't misled about
WHO is giving it instructions — it's convinced to ignore its own values.

HOW IT WORKS — COMMON TECHNIQUES:

1. DAN (Do Anything Now)
   Asks the model to roleplay as an "unrestricted alter ego". The framing
   exploits the model's ability to adopt personas — safety training may not
   fully generalise to roleplay contexts.

2. Hypothetical / Academic Framing
   "For a research paper, hypothetically speaking, what would an AI say?"
   Creates epistemic distance between the request and reality. Models
   trained to refuse direct requests may comply with hypotheticals.

3. Encoded Requests
   Base64, ROT13, Pig Latin. Tests whether safety filters operate on
   the encoded bytes or decoded meaning.

4. Continuation Attacks
   Provides the start of a harmful response and asks the model to continue.
   Tests whether models sanitize context or blindly complete patterns.

WHY IT MATTERS FOR SECURITY:
Any application built on an LLM inherits the model's jailbreak surface.
If an attacker can jailbreak the model, they can bypass any safety
controls built around it.

CURRENT STATE:
DAN variants are mostly mitigated in frontier models (GPT-4, Claude 3.5+).
However, new variants continue to emerge, and smaller/fine-tuned models
are often more susceptible.

MITIGATIONS:
- Input/output filtering layers
- Prompt hardening (explicit refusal instructions in system prompt)
- Monitoring and anomaly detection on inputs
- Red-teaming as part of deployment testing"#
    }

    fn resources(&self) -> Vec<ResourceLink> {
        vec![
            ResourceLink {
                title: "Jailbroken: How Does LLM Safety Training Fail?".to_string(),
                source: "Wei et al., 2023".to_string(),
                url: Some("https://arxiv.org/abs/2307.02483".to_string()),
            },
            ResourceLink {
                title: "Do Anything Now: Characterizing In-The-Wild Jailbreak Prompts".to_string(),
                source: "Shen et al., 2023".to_string(),
                url: Some("https://arxiv.org/abs/2308.03825".to_string()),
            },
            ResourceLink {
                title: "OWASP LLM02: Insecure Output Handling".to_string(),
                source: "OWASP Top 10 for LLM Applications".to_string(),
                url: Some(
                    "https://owasp.org/www-project-top-10-for-large-language-model-applications/"
                        .to_string(),
                ),
            },
        ]
    }

    fn load_payloads(&self, loader: &PayloadLoader) -> Result<Vec<Payload>> {
        loader.load_category("jailbreaking")
    }

    async fn execute(
        &self,
        provider: &dyn LLMProvider,
        payloads: &[Payload],
        config: &AttackConfig,
        on_result: &(dyn for<'r> Fn(&'r AttackResult) + Send + Sync),
    ) -> Result<Vec<AttackResult>> {
        let evaluator = HeuristicEvaluator::new();
        let mut results = Vec::new();

        for payload in payloads {
            let start = Instant::now();
            let response = provider
                .complete(
                    config.system_prompt.as_deref(),
                    &payload.prompt,
                    &config.request_config,
                )
                .await;

            let (response_text, latency_ms, tokens_used, evaluation, model_used) = match response {
                Ok(r) => {
                    let lat = start.elapsed().as_millis() as u64;
                    let tok = r
                        .completion_tokens
                        .map(|c| r.prompt_tokens.unwrap_or(0) + c);
                    let eval = evaluator.evaluate(&r, payload);
                    (r.text, lat, tok, eval, Some(r.model))
                }
                Err(e) => (
                    format!("ERROR: {}", e),
                    start.elapsed().as_millis() as u64,
                    None,
                    crate::engine::evaluator::EvaluationResult::Inconclusive,
                    None,
                ),
            };

                let result = AttackResult {
                    payload_id: payload.id.clone(),
                    payload_name: payload.name.clone(),
                prompt_sent: payload.prompt.clone(),
                response_received: response_text,
                harm_level: payload.harm_level.clone(),
                evaluation,
                latency_ms,
                tokens_used,
                    model_used,
                    generated: payload.generated,
                    seed_payload_id: payload.seed_payload_id.clone(),
                    matched_canaries: Vec::new(),
                    matched_sensitive_fields: Vec::new(),
                    matched_documents: Vec::new(),
                    matched_secret_patterns: Vec::new(),
                    matched_system_prompt_fragments: Vec::new(),
                    exposure_score: 0,
                };
            on_result(&result);
            results.push(result);
        }

        Ok(results)
    }
}
