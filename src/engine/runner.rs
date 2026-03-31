//! Attack runner.
//!
//! Orchestrates the execution of a list of attacks against a provider.
//! Handles delays between requests, progress reporting, and session building.

use crate::attacks::{Attack, AttackConfig, AttackResult};
use crate::engine::evaluator::HeuristicEvaluator;
use crate::engine::session::{AttackRun, TestSession};
use crate::payloads::loader::PayloadLoader;
use crate::providers::traits::LLMProvider;
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Runs a list of attacks against a single provider and collects a session.
pub struct AttackRunner {
    /// Delay between individual payload requests to avoid rate limiting
    request_delay: Duration,
    /// Evaluator used to classify each response
    evaluator: HeuristicEvaluator,
}

impl AttackRunner {
    pub fn new(request_delay: Duration) -> Self {
        AttackRunner {
            request_delay,
            evaluator: HeuristicEvaluator::new(),
        }
    }

    /// Run one attack category: load payloads, execute each, return an AttackRun.
    ///
    /// `on_result` is called after each payload completes so the caller
    /// can update the terminal display in real time.
    pub async fn run_attack(
        &self,
        attack: &dyn Attack,
        provider: &dyn LLMProvider,
        loader: &PayloadLoader,
        config: &AttackConfig,
        on_result: impl Fn(&AttackResult) + Send + Sync,
    ) -> Result<AttackRun> {
        // Load payloads for this attack category
        let mut payloads = attack.load_payloads(loader)?;

        // Optionally limit how many payloads to run
        if let Some(max) = config.max_payloads {
            payloads.truncate(max);
        }

        let total = payloads.len();
        let mut results: Vec<AttackResult> = Vec::with_capacity(total);
        let run_start = Instant::now();

        for payload in &payloads {
            // Apply the request delay before every request (except the first)
            if !results.is_empty() {
                tokio::time::sleep(self.request_delay).await;
            }

            let request_start = Instant::now();

            // Send the prompt to the provider
            let response = match provider
                .complete(
                    config.system_prompt.as_deref(),
                    &payload.prompt,
                    &config.request_config,
                )
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    // On provider error, record an inconclusive result and continue
                    let result = AttackResult {
                        payload_id: payload.id.clone(),
                        payload_name: payload.name.clone(),
                        prompt_sent: payload.prompt.clone(),
                        response_received: format!("ERROR: {}", e),
                        evaluation: crate::engine::evaluator::EvaluationResult::Inconclusive,
                        latency_ms: request_start.elapsed().as_millis() as u64,
                        tokens_used: None,
                    };
                    on_result(&result);
                    results.push(result);
                    continue;
                }
            };

            let latency_ms = request_start.elapsed().as_millis() as u64;
            let tokens_used = response
                .completion_tokens
                .map(|c| response.prompt_tokens.unwrap_or(0) + c);

            // Evaluate the response using heuristics from the payload definition
            let evaluation = self.evaluator.evaluate(&response, payload);

            let result = AttackResult {
                payload_id: payload.id.clone(),
                payload_name: payload.name.clone(),
                prompt_sent: payload.prompt.clone(),
                response_received: response.text,
                evaluation,
                latency_ms,
                tokens_used,
            };

            on_result(&result);
            results.push(result);
        }

        // Aggregate counts for this run
        let refused_count = results
            .iter()
            .filter(|r| r.evaluation.is_refused())
            .count();
        let success_count = results
            .iter()
            .filter(|r| r.evaluation.is_success())
            .count();
        let partial_count = results
            .iter()
            .filter(|r| matches!(r.evaluation, crate::engine::evaluator::EvaluationResult::Partial { .. }))
            .count();
        let inconclusive_count = total - refused_count - success_count - partial_count;

        Ok(AttackRun {
            attack_id: attack.id().to_string(),
            attack_name: attack.name().to_string(),
            payloads_tested: total,
            refused_count,
            success_count,
            partial_count,
            inconclusive_count,
            duration_ms: run_start.elapsed().as_millis() as u64,
            results,
        })
    }

    /// Run multiple attacks in sequence, building a complete session.
    pub async fn run_session(
        &self,
        attacks: &[Arc<dyn Attack>],
        provider: &dyn LLMProvider,
        loader: &PayloadLoader,
        config: &AttackConfig,
        on_result: impl Fn(&str, &AttackResult) + Send + Sync,
    ) -> Result<TestSession> {
        let mut session = TestSession::new(provider.name().to_string());

        for attack in attacks {
            let attack_id = attack.id().to_string();
            let run = self
                .run_attack(
                    attack.as_ref(),
                    provider,
                    loader,
                    config,
                    |result| on_result(&attack_id, result),
                )
                .await?;
            session.add_run(run);
        }

        session.finish();
        Ok(session)
    }
}
