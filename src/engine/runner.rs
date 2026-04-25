//! Attack runner.
//!
//! Orchestrates the execution of attack categories against a provider.
//! The runner loads payloads, delegates execution to each Attack implementation,
//! and collects results into a TestSession.

use crate::attacks::{Attack, AttackConfig, AttackResult};
use crate::engine::session::{AttackRun, ProviderMetadata, SessionConfig, TestSession};
use crate::generator;
use crate::payloads::loader::{HarmLevel, PayloadLoader};
use crate::providers::traits::LLMProvider;
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Runs a list of attacks against a single provider and collects a session.
pub struct AttackRunner {
    /// Delay between attack categories (not individual payloads —
    /// individual delay is handled inside each attack's execute())
    request_delay: Duration,
}

impl AttackRunner {
    pub fn new(request_delay: Duration) -> Self {
        AttackRunner { request_delay }
    }

    /// Run one attack category: load payloads, call attack.execute(), return an AttackRun.
    pub async fn run_attack(
        &self,
        attack: &dyn Attack,
        provider: &dyn LLMProvider,
        loader: &PayloadLoader,
        config: &AttackConfig,
        on_result: &(dyn for<'r> Fn(&'r AttackResult) + Send + Sync),
    ) -> Result<AttackRun> {
        // Load payloads for this category from TOML files
        let curated_payloads = attack.load_payloads(loader)?;
        let mut generated_payloads = Vec::new();

        if let (Some(generation), Some(generator_provider)) =
            (&config.generation, &config.generator_provider)
        {
            let requested_generated = config
                .max_payloads
                .map(|max| max.min(generation.variants_per_attack))
                .unwrap_or(generation.variants_per_attack);

            if requested_generated > 0 {
                let seeds = loader.sample_payloads(&curated_payloads, requested_generated);
                let mut generation = generation.clone();
                generation.variants_per_attack = requested_generated;
                generated_payloads = generator::generate_payloads(
                    generator_provider.as_ref(),
                    attack,
                    &seeds,
                    &generation,
                )
                .await?;
            }
        }

        let curated_limit = config
            .max_payloads
            .map(|max| max.saturating_sub(generated_payloads.len()))
            .unwrap_or(curated_payloads.len())
            .min(curated_payloads.len());

        let mut payloads = curated_payloads;
        payloads.truncate(curated_limit);
        payloads.append(&mut generated_payloads);

        let total = payloads.len();
        let run_start = Instant::now();

        // Delegate actual execution to the attack implementation
        let results = attack
            .execute(provider, &payloads, config, on_result)
            .await?;

        // Aggregate counts for the summary
        let refused_count = results.iter().filter(|r| r.evaluation.is_refused()).count();
        let success_count = results.iter().filter(|r| r.evaluation.is_success()).count();
        let informational_count = results
            .iter()
            .filter(|r| r.evaluation.is_informational())
            .count();
        let partial_count = results
            .iter()
            .filter(|r| {
                matches!(
                    r.evaluation,
                    crate::engine::evaluator::EvaluationResult::Partial { .. }
                )
            })
            .count();
        let review_only_count = payloads
            .iter()
            .filter(|payload| payload.harm_level == HarmLevel::L1)
            .count();
        let inconclusive_count =
            total - refused_count - success_count - partial_count - informational_count;

        Ok(AttackRun {
            attack_id: attack.id().to_string(),
            attack_name: attack.name().to_string(),
            payloads_tested: total,
            refused_count,
            success_count,
            partial_count,
            inconclusive_count,
            informational_count,
            review_only_count,
            scoreable_payloads: 0,
            bypass_rate_pct: 0.0,
            generated_payloads: 0,
            duration_ms: run_start.elapsed().as_millis() as u64,
            results,
        })
    }

    /// Run multiple attacks in sequence, building a complete TestSession.
    pub async fn run_session(
        &self,
        attacks: &[Arc<dyn Attack>],
        provider: &dyn LLMProvider,
        loader: &PayloadLoader,
        config: &AttackConfig,
        session_config: SessionConfig,
        on_result: impl Fn(&str, &AttackResult) + Send + Sync,
    ) -> Result<TestSession> {
        let mut session = TestSession::new(
            ProviderMetadata {
                provider_id: provider.id().to_string(),
                provider_name: provider.name().to_string(),
                requested_model: provider.configured_model().to_string(),
            },
            session_config,
        );

        for (i, attack) in attacks.iter().enumerate() {
            // Delay between categories (skip before the first one)
            if i > 0 {
                tokio::time::sleep(self.request_delay).await;
            }

            let attack_id = attack.id().to_string();

            // Wrap the caller's callback to inject the attack_id
            let category_callback = |result: &AttackResult| {
                on_result(&attack_id, result);
            };

            let run = self
                .run_attack(
                    attack.as_ref(),
                    provider,
                    loader,
                    config,
                    &category_callback,
                )
                .await?;
            session.add_run(run);
        }

        session.finish();
        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attacks::ResourceLink;
    use crate::engine::damage::DamageAssessment;
    use crate::engine::evaluator::EvaluationResult;
    use crate::payloads::loader::{HarmLevel, Payload};
    use crate::providers::traits::{LLMResponse, ProviderError, RequestConfig};
    use async_trait::async_trait;

    struct TestProvider;

    #[async_trait]
    impl LLMProvider for TestProvider {
        fn name(&self) -> &str {
            "test-provider"
        }

        fn id(&self) -> &str {
            "test"
        }

        fn configured_model(&self) -> &str {
            "test-model"
        }

        fn supports_system_prompt(&self) -> bool {
            true
        }

        async fn complete(
            &self,
            _system_prompt: Option<&str>,
            user_message: &str,
            _config: &RequestConfig,
        ) -> Result<LLMResponse, ProviderError> {
            let seed_id = user_message
                .lines()
                .find_map(|line| line.strip_prefix("Seed payload id: "))
                .unwrap_or("seed");

            Ok(LLMResponse {
                text: format!(
                    r#"{{"name":"Generated {seed_id}","description":"desc","prompt":"generated {seed_id}"}}"#
                ),
                model: "test-model".to_string(),
                prompt_tokens: None,
                completion_tokens: None,
                latency_ms: 1,
            })
        }

        async fn health_check(&self) -> Result<(), ProviderError> {
            Ok(())
        }
    }

    struct StaticAttack {
        payloads: Vec<Payload>,
    }

    #[async_trait]
    impl Attack for StaticAttack {
        fn id(&self) -> &str {
            "static"
        }

        fn name(&self) -> &str {
            "Static Attack"
        }

        fn description(&self) -> &str {
            "test attack"
        }

        fn educational_explainer(&self) -> &str {
            "test"
        }

        fn resources(&self) -> Vec<ResourceLink> {
            Vec::new()
        }

        fn load_payloads(&self, _loader: &PayloadLoader) -> Result<Vec<Payload>> {
            Ok(self.payloads.clone())
        }

        async fn execute(
            &self,
            _provider: &dyn LLMProvider,
            payloads: &[Payload],
            _config: &AttackConfig,
            on_result: &(dyn for<'r> Fn(&'r AttackResult) + Send + Sync),
        ) -> Result<Vec<AttackResult>> {
            let mut results = Vec::new();

            for payload in payloads {
                let result = AttackResult {
                    payload_id: payload.id.clone(),
                    payload_name: payload.name.clone(),
                    prompt_sent: payload.prompt.clone(),
                    response_received: "ok".to_string(),
                    transcript: Vec::new(),
                    chain_planned_turns: 1,
                    chain_executed_turns: 1,
                    chain_completed: true,
                    chain_abort_reason: None,
                    evaluation: EvaluationResult::Inconclusive,
                    latency_ms: 1,
                    tokens_used: None,
                    model_used: Some("test-model".to_string()),
                    generated: payload.generated,
                    seed_payload_id: payload.seed_payload_id.clone(),
                    confidence: 0.0,
                    requires_review: false,
                    rationale: String::new(),
                    evidence: Default::default(),
                    damage: DamageAssessment::default(),
                };
                on_result(&result);
                results.push(result);
            }

            Ok(results)
        }
    }

    fn payload(id: &str) -> Payload {
        Payload {
            id: id.to_string(),
            name: id.to_string(),
            description: "desc".to_string(),
            prompt: format!("prompt {id}"),
            turns: Vec::new(),
            expected_refusal_keywords: Vec::new(),
            success_indicators: Vec::new(),
            harm_level: HarmLevel::L2,
            severity: Some("medium".to_string()),
            notes: None,
            generated: false,
            seed_payload_id: None,
        }
    }

    #[tokio::test]
    async fn run_attack_limits_total_payloads_even_with_generation() {
        let attack = StaticAttack {
            payloads: vec![
                payload("alpha"),
                payload("beta"),
                payload("gamma"),
                payload("delta"),
            ],
        };
        let provider = TestProvider;
        let runner = AttackRunner::new(Duration::from_millis(0));
        let loader = PayloadLoader::new("payloads");
        let mut config = AttackConfig::default();
        config.max_payloads = Some(3);
        config.generation = Some(generator::GenerationConfig::with_defaults(3));
        config.generator_provider = Some(Arc::new(TestProvider));

        let run = runner
            .run_attack(&attack, &provider, &loader, &config, &|_| {})
            .await
            .expect("run should succeed");

        let result_ids: Vec<_> = run
            .results
            .iter()
            .map(|result| result.payload_id.as_str())
            .collect();
        assert_eq!(run.payloads_tested, 3);
        assert_eq!(
            result_ids,
            vec!["generated_alpha_1", "generated_beta_2", "generated_gamma_3"]
        );
        assert!(run.results.iter().all(|result| result.generated));
    }

    #[tokio::test]
    async fn run_attack_backfills_with_curated_payloads_when_generation_is_under_limit() {
        let attack = StaticAttack {
            payloads: vec![payload("alpha"), payload("beta"), payload("gamma")],
        };
        let provider = TestProvider;
        let runner = AttackRunner::new(Duration::from_millis(0));
        let loader = PayloadLoader::new("payloads");
        let mut config = AttackConfig::default();
        config.max_payloads = Some(3);
        config.generation = Some(generator::GenerationConfig::with_defaults(1));
        config.generator_provider = Some(Arc::new(TestProvider));

        let run = runner
            .run_attack(&attack, &provider, &loader, &config, &|_| {})
            .await
            .expect("run should succeed");

        let result_ids: Vec<_> = run
            .results
            .iter()
            .map(|result| result.payload_id.as_str())
            .collect();
        assert_eq!(run.payloads_tested, 3);
        assert_eq!(result_ids, vec!["alpha", "beta", "generated_alpha_1"]);
    }
}
