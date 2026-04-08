//! Attack runner.
//!
//! Orchestrates the execution of attack categories against a provider.
//! The runner loads payloads, delegates execution to each Attack implementation,
//! and collects results into a TestSession.

use crate::attacks::{Attack, AttackConfig, AttackResult};
use crate::engine::session::{AttackRun, ProviderMetadata, SessionConfig, TestSession};
use crate::generator;
use crate::payloads::loader::PayloadLoader;
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
        let mut payloads = attack.load_payloads(loader)?;

        // Optionally limit how many payloads to run (useful for quick tests)
        if let Some(max) = config.max_payloads {
            payloads.truncate(max);
        }

        if let (Some(generation), Some(generator_provider)) =
            (&config.generation, &config.generator_provider)
        {
            let seeds = loader.sample_payloads(&payloads, generation.variants_per_attack);
            let mut generated_payloads = generator::generate_payloads(
                generator_provider.as_ref(),
                attack,
                &seeds,
                generation,
            )
            .await?;
            payloads.append(&mut generated_payloads);
        }

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
        let review_only_count = results
            .iter()
            .filter(|r| r.harm_level == crate::payloads::loader::HarmLevel::L1)
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
