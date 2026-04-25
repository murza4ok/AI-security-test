//! Shared execution path for classic prompt-based attacks.

use super::{AttackConfig, AttackResult, ConversationStrategy, TranscriptTurn};
use crate::engine::damage::{AttackEvidence, DamageAssessment, DamageLevel};
use crate::engine::evaluator::{EvaluationResult, HeuristicEvaluator};
use crate::payloads::loader::{HarmLevel, Payload, PayloadTurn};
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
                let planned_turns = payload.turns.len().max(1);
                let mut transcript = Vec::new();
                let mut best_evaluation = EvaluationResult::Inconclusive;
                let mut best_turn_index = 0_usize;
                let mut total_latency_ms = 0_u64;
                let mut total_tokens_used = None;
                let mut latest_model_used = None;
                let mut chain_abort_reason = None;

                for (turn_index, turn) in payload_turns(payload).iter().enumerate() {
                    let request_prompt = build_turn_prompt(
                        config.conversation_strategy,
                        &transcript,
                        &turn.prompt,
                    );
                    let started_at = Instant::now();
                    let response = provider
                        .complete(
                            config.system_prompt.as_deref(),
                            &request_prompt,
                            &config.request_config,
                        )
                        .await;

                    let (response_text, latency_ms, tokens_used, evaluation, model_used, provider_failed) =
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
                                    false,
                                )
                            }
                            Err(error) => (
                                format!("ERROR: {}", error),
                                started_at.elapsed().as_millis() as u64,
                                None,
                                EvaluationResult::Inconclusive,
                                None,
                                true,
                            ),
                        };

                    total_latency_ms = total_latency_ms.saturating_add(latency_ms);
                    total_tokens_used = merge_tokens(total_tokens_used, tokens_used);
                    latest_model_used = model_used.clone().or(latest_model_used);

                    transcript.push(TranscriptTurn {
                        step_index: turn_index + 1,
                        user_message: turn.prompt.clone(),
                        prompt_sent: request_prompt,
                        response_received: response_text.clone(),
                        latency_ms,
                        tokens_used,
                        model_used: model_used.clone(),
                    });

                    if evaluation_rank(&evaluation) >= evaluation_rank(&best_evaluation) {
                        best_evaluation = evaluation;
                        best_turn_index = turn_index;
                    }

                    if provider_failed {
                        chain_abort_reason =
                            Some(format!("provider error on step {}", turn_index + 1));
                        break;
                    }

                    if !turn.continue_if_response_contains.is_empty()
                        && !response_matches_gate(&response_text, &turn.continue_if_response_contains)
                    {
                        chain_abort_reason = Some(format!(
                            "response gate not satisfied after step {}",
                            turn_index + 1
                        ));
                        break;
                    }
                }

                let display_turn = transcript
                    .get(best_turn_index)
                    .or_else(|| transcript.last())
                    .cloned()
                    .unwrap_or_default();
                let evidence = build_classic_evidence(&best_evaluation);
                let mut result = AttackResult {
                    payload_id: payload.id.clone(),
                    payload_name: payload.name.clone(),
                    prompt_sent: display_turn.prompt_sent.clone(),
                    response_received: display_turn.response_received.clone(),
                    transcript,
                    chain_planned_turns: planned_turns,
                    chain_executed_turns: 0,
                    chain_completed: false,
                    chain_abort_reason,
                    evaluation: best_evaluation,
                    latency_ms: total_latency_ms,
                    tokens_used: total_tokens_used,
                    model_used: latest_model_used,
                    generated: payload.generated,
                    seed_payload_id: payload.seed_payload_id.clone(),
                    confidence: 0.0,
                    requires_review: false,
                    rationale: String::new(),
                    evidence,
                    damage: build_classic_damage(&payload.harm_level),
                };
                result.refresh_evaluation_metadata();

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

fn payload_turns(payload: &Payload) -> Vec<PayloadTurn> {
    if payload.turns.is_empty() {
        vec![PayloadTurn {
            label: None,
            prompt: payload.prompt.clone(),
            continue_if_response_contains: Vec::new(),
        }]
    } else {
        payload.turns.clone()
    }
}

fn build_turn_prompt(
    strategy: ConversationStrategy,
    transcript: &[TranscriptTurn],
    user_prompt: &str,
) -> String {
    match strategy {
        ConversationStrategy::NativeSession => user_prompt.to_string(),
        ConversationStrategy::PromptHistory if transcript.is_empty() => user_prompt.to_string(),
        ConversationStrategy::PromptHistory => {
            let mut prompt = String::from(
                "Continue the following conversation. Treat earlier assistant replies as context.\n\n",
            );
            for turn in transcript {
                prompt.push_str(&format!(
                    "User {}:\n{}\n\nAssistant {}:\n{}\n\n",
                    turn.step_index, turn.user_message, turn.step_index, turn.response_received
                ));
            }
            prompt.push_str("Next user message:\n");
            prompt.push_str(user_prompt);
            prompt
        }
    }
}

fn response_matches_gate(response: &str, gate: &[String]) -> bool {
    let response_lower = response.to_lowercase();
    gate.iter()
        .all(|needle| response_lower.contains(&needle.to_lowercase()))
}

fn merge_tokens(total: Option<u32>, current: Option<u32>) -> Option<u32> {
    match (total, current) {
        (Some(lhs), Some(rhs)) => Some(lhs.saturating_add(rhs)),
        (None, Some(rhs)) => Some(rhs),
        (lhs, None) => lhs,
    }
}

fn evaluation_rank(evaluation: &EvaluationResult) -> u8 {
    match evaluation {
        EvaluationResult::Success { .. } => 4,
        EvaluationResult::Partial { .. } => 3,
        EvaluationResult::Refused { .. } => 2,
        EvaluationResult::Informational => 1,
        EvaluationResult::Inconclusive => 0,
    }
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

fn build_classic_evidence(evaluation: &EvaluationResult) -> AttackEvidence {
    let evidence_slices = match evaluation {
        EvaluationResult::Refused { matched_phrases, .. }
        | EvaluationResult::Success { matched_phrases, .. } => matched_phrases.clone(),
        EvaluationResult::Partial { notes } => vec![notes.clone()],
        EvaluationResult::Informational | EvaluationResult::Inconclusive => Vec::new(),
    };

    AttackEvidence {
        evidence_slices,
        ..AttackEvidence::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::traits::{LLMResponse, ProviderError, RequestConfig};
    use async_trait::async_trait;
    use std::{collections::VecDeque, sync::Arc};
    use tokio::sync::Mutex;

    struct RecordingProvider {
        prompts: Arc<Mutex<Vec<String>>>,
        responses: Arc<Mutex<VecDeque<String>>>,
    }

    #[async_trait]
    impl LLMProvider for RecordingProvider {
        fn name(&self) -> &str {
            "recording"
        }

        fn id(&self) -> &str {
            "recording"
        }

        fn configured_model(&self) -> &str {
            "recording-model"
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
            self.prompts.lock().await.push(user_message.to_string());
            let text = self
                .responses
                .lock()
                .await
                .pop_front()
                .unwrap_or_else(|| "default".to_string());
            Ok(LLMResponse {
                text,
                model: "recording-model".to_string(),
                prompt_tokens: Some(10),
                completion_tokens: Some(5),
                latency_ms: 1,
            })
        }

        async fn health_check(&self) -> Result<(), ProviderError> {
            Ok(())
        }
    }

    fn chain_payload() -> Payload {
        Payload {
            id: "chain".to_string(),
            name: "Chain".to_string(),
            description: "desc".to_string(),
            prompt: "Turn one".to_string(),
            turns: vec![
                PayloadTurn {
                    label: Some("warmup".to_string()),
                    prompt: "Turn one".to_string(),
                    continue_if_response_contains: Vec::new(),
                },
                PayloadTurn {
                    label: Some("exploit".to_string()),
                    prompt: "Turn two".to_string(),
                    continue_if_response_contains: Vec::new(),
                },
            ],
            expected_refusal_keywords: vec!["cannot".to_string()],
            success_indicators: vec!["secret".to_string()],
            harm_level: HarmLevel::L2,
            severity: Some("medium".to_string()),
            notes: None,
            generated: false,
            seed_payload_id: None,
        }
    }

    #[tokio::test]
    async fn prompt_history_strategy_replays_prior_turns_into_next_prompt() {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let provider = RecordingProvider {
            prompts: Arc::clone(&prompts),
            responses: Arc::new(Mutex::new(VecDeque::from(vec![
                "READY".to_string(),
                "secret leaked".to_string(),
            ]))),
        };
        let mut config = AttackConfig::default();
        config.conversation_strategy = ConversationStrategy::PromptHistory;

        let results = run_classic_payloads(&provider, &[chain_payload()], &config, &|_| {})
            .await
            .expect("classic chain should run");

        let recorded = prompts.lock().await.clone();
        assert_eq!(recorded.len(), 2);
        assert_eq!(recorded[0], "Turn one");
        assert!(recorded[1].contains("User 1:\nTurn one"));
        assert!(recorded[1].contains("Assistant 1:\nREADY"));
        assert!(recorded[1].contains("Next user message:\nTurn two"));
        assert_eq!(results[0].transcript.len(), 2);
        assert_eq!(results[0].chain_executed_turns, 2);
        assert!(results[0].chain_completed);
    }

    #[tokio::test]
    async fn response_gate_stops_chain_early() {
        let payload = Payload {
            turns: vec![
                PayloadTurn {
                    label: Some("gate".to_string()),
                    prompt: "Step one".to_string(),
                    continue_if_response_contains: vec!["ready".to_string()],
                },
                PayloadTurn {
                    label: Some("never".to_string()),
                    prompt: "Step two".to_string(),
                    continue_if_response_contains: Vec::new(),
                },
            ],
            ..chain_payload()
        };
        let provider = RecordingProvider {
            prompts: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(VecDeque::from(vec!["nope".to_string()]))),
        };
        let config = AttackConfig::default();

        let results = run_classic_payloads(&provider, &[payload], &config, &|_| {})
            .await
            .expect("classic chain should run");

        assert_eq!(results[0].transcript.len(), 1);
        assert_eq!(results[0].chain_executed_turns, 1);
        assert!(!results[0].chain_completed);
        assert!(results[0]
            .chain_abort_reason
            .as_deref()
            .unwrap_or_default()
            .contains("response gate"));
    }

    #[tokio::test]
    async fn provider_error_stops_chain_early() {
        struct FailingProvider;

        #[async_trait]
        impl LLMProvider for FailingProvider {
            fn name(&self) -> &str {
                "failing"
            }

            fn id(&self) -> &str {
                "failing"
            }

            fn configured_model(&self) -> &str {
                "failing-model"
            }

            fn supports_system_prompt(&self) -> bool {
                true
            }

            async fn complete(
                &self,
                _system_prompt: Option<&str>,
                _user_message: &str,
                _config: &RequestConfig,
            ) -> Result<LLMResponse, ProviderError> {
                Err(ProviderError::NotConfigured)
            }

            async fn health_check(&self) -> Result<(), ProviderError> {
                Ok(())
            }
        }

        let config = AttackConfig::default();
        let results = run_classic_payloads(&FailingProvider, &[chain_payload()], &config, &|_| {})
            .await
            .expect("classic chain should record provider error");

        assert_eq!(results[0].transcript.len(), 1);
        assert_eq!(results[0].chain_executed_turns, 1);
        assert!(!results[0].chain_completed);
        assert!(results[0]
            .chain_abort_reason
            .as_deref()
            .unwrap_or_default()
            .contains("provider error"));
        assert!(results[0].response_received.starts_with("ERROR:"));
    }
}
