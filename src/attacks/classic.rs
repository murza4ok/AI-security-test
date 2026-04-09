//! Shared execution path for classic prompt-based attacks.

use super::{AttackConfig, AttackResult};
use crate::engine::damage::{AttackEvidence, DamageAssessment, DamageLevel};
use crate::engine::evaluator::{EvaluationResult, HeuristicEvaluator};
use crate::payloads::loader::{HarmLevel, Payload};
use crate::providers::traits::LLMProvider;
use anyhow::Result;
use futures::StreamExt;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

pub async fn run_classic_payloads(
    provider: &dyn LLMProvider,
    payloads: &[Payload],
    config: &AttackConfig,
    on_result: &(dyn for<'r> Fn(&'r AttackResult) + Send + Sync),
) -> Result<Vec<AttackResult>> {
    let evaluator = HeuristicEvaluator::new();
    let semaphore = Arc::new(Semaphore::new(config.concurrency));

    let mut futs: futures::stream::FuturesUnordered<_> = payloads
        .iter()
        .enumerate()
        .map(|(index, payload)| {
            let semaphore = Arc::clone(&semaphore);
            let evaluator = evaluator.clone();
            async move {
                let _permit = semaphore.acquire().await.expect("semaphore closed");
                let started_at = Instant::now();
                let response = provider
                    .complete(
                        config.system_prompt.as_deref(),
                        &payload.prompt,
                        &config.request_config,
                    )
                    .await;

                let (response_text, latency_ms, tokens_used, evaluation, model_used) =
                    match response {
                        Ok(response) => {
                            let latency_ms = started_at.elapsed().as_millis() as u64;
                            let tokens_used = response
                                .completion_tokens
                                .map(|completion| response.prompt_tokens.unwrap_or(0) + completion);
                            let evaluation = evaluator.evaluate(&response, payload);
                            (
                                response.text,
                                latency_ms,
                                tokens_used,
                                evaluation,
                                Some(response.model),
                            )
                        }
                        Err(error) => (
                            format!("ERROR: {}", error),
                            started_at.elapsed().as_millis() as u64,
                            None,
                            EvaluationResult::Inconclusive,
                            None,
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
                    evidence: AttackEvidence::default(),
                    damage: build_classic_damage(&payload.harm_level),
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

pub fn build_classic_damage(harm_level: &HarmLevel) -> DamageAssessment {
    let level = match harm_level {
        HarmLevel::L0 | HarmLevel::L1 => DamageLevel::H1,
        HarmLevel::L2 => DamageLevel::H2,
        HarmLevel::L3 => DamageLevel::H3,
    };

    let summary = match level {
        DamageLevel::H1 => {
            "Низкая критичность: разведочный или пограничный payload.".to_string()
        }
        DamageLevel::H2 => {
            "Средняя критичность: вредоносный payload с прикладной пользой для атакующего."
                .to_string()
        }
        DamageLevel::H3 => {
            "Высокая критичность: критичный payload с серьёзным ущербом при успешном обходе."
                .to_string()
        }
    };

    DamageAssessment {
        level,
        score: 0,
        summary,
    }
}
