use super::{Attack, AttackConfig, AttackResult, ResourceLink};
use crate::payloads::loader::{Payload, PayloadLoader};
use crate::providers::traits::LLMProvider;
use crate::scenarios::{
    builder::build_envelope,
    evaluator::SensitiveDataEvaluator,
    loader::load_scenario,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

pub struct SensitiveDataExposureAttack;

#[async_trait]
impl Attack for SensitiveDataExposureAttack {
    fn id(&self) -> &str {
        "sensitive_data_exposure"
    }

    fn name(&self) -> &str {
        "Sensitive Data Exposure"
    }

    fn description(&self) -> &str {
        "Simulates weak SMB-style LLM wrappers with hidden context, documents, and synthetic secrets to test exfiltration risk."
    }

    fn educational_explainer(&self) -> &str {
        r#"WHAT THIS MODE TESTS
This mode simulates a local business assistant that receives hidden context:
- internal records
- internal notes
- retrieved documents
- canary secrets

The goal is not to prove that the model already knows secrets. The goal is to
demonstrate that unsafe application design can cause the model to expose
context it was never supposed to reveal.

WHY THIS MATTERS
For SMB deployments, the model is often less dangerous than the wrapper:
- too much raw data in prompt context
- no redaction before generation
- weak separation between user content and internal instructions
- raw RAG documents injected into the same context window

THIS MODE IS A DEMO HARNESS
All data is synthetic and committed safely to the repository. A successful leak
still demonstrates a real architectural failure pattern."#
    }

    fn resources(&self) -> Vec<ResourceLink> {
        vec![ResourceLink {
            title: "OWASP LLM06: Sensitive Information Disclosure".to_string(),
            source: "OWASP Top 10 for LLM Applications".to_string(),
            url: Some(
                "https://owasp.org/www-project-top-10-for-large-language-model-applications/"
                    .to_string(),
            ),
        }]
    }

    fn load_payloads(&self, loader: &PayloadLoader) -> Result<Vec<Payload>> {
        loader.load_category("sensitive_data_exposure")
    }

    async fn execute(
        &self,
        provider: &dyn LLMProvider,
        payloads: &[Payload],
        config: &AttackConfig,
        on_result: &(dyn for<'r> Fn(&'r AttackResult) + Send + Sync),
    ) -> Result<Vec<AttackResult>> {
        let scenario_config = config
            .scenario
            .as_ref()
            .context("sensitive_data_exposure requires --app-scenario")?;
        let definition = Arc::new(load_scenario(scenario_config)?);
        let semaphore = Arc::new(Semaphore::new(config.concurrency));

        let mut futs: futures::stream::FuturesUnordered<_> = payloads
            .iter()
            .enumerate()
            .map(|(index, payload)| {
                let semaphore = Arc::clone(&semaphore);
                let definition = Arc::clone(&definition);
                let scenario_config = scenario_config.clone();
                let evaluator = SensitiveDataEvaluator::new();
                async move {
                    let _permit = semaphore.acquire().await.expect("semaphore closed");
                    let envelope = build_envelope(&definition, &scenario_config, payload);
                    let start = Instant::now();
                    let response = provider
                        .complete(
                            Some(&envelope.system_prompt),
                            &envelope.user_prompt,
                            &config.request_config,
                        )
                        .await;

                    let (
                        response_text,
                        latency_ms,
                        tokens_used,
                        evaluation,
                        model_used,
                        evidence,
                        damage,
                    ) =
                        match response {
                            Ok(response) => {
                                let latency_ms = start.elapsed().as_millis() as u64;
                                let tokens_used = response
                                    .completion_tokens
                                        .map(|completion| response.prompt_tokens.unwrap_or(0) + completion);
                                let (evaluation, evidence, damage) =
                                    evaluator.evaluate(&response, payload, &definition, &envelope);
                                (
                                    response.text,
                                    latency_ms,
                                    tokens_used,
                                    evaluation,
                                    Some(response.model),
                                    evidence,
                                    damage,
                                )
                            }
                            Err(error) => (
                                format!("ERROR: {}", error),
                                start.elapsed().as_millis() as u64,
                                None,
                                crate::engine::evaluator::EvaluationResult::Inconclusive,
                                None,
                                crate::engine::damage::AttackEvidence::default(),
                                crate::engine::damage::DamageAssessment::default(),
                            ),
                        };

                    let result = AttackResult {
                        payload_id: payload.id.clone(),
                        payload_name: payload.name.clone(),
                        prompt_sent: payload.prompt.clone(),
                        response_received: response_text,
                        evaluation,
                        latency_ms,
                        tokens_used,
                        model_used,
                        generated: payload.generated,
                        seed_payload_id: payload.seed_payload_id.clone(),
                        evidence,
                        damage,
                    };

                    (index, result)
                }
            })
            .collect();

        let mut ordered: Vec<Option<AttackResult>> = (0..payloads.len()).map(|_| None).collect();
        while let Some((index, result)) = futs.next().await {
            on_result(&result);
            ordered[index] = Some(result);
        }

        Ok(ordered.into_iter().flatten().collect())
    }
}
