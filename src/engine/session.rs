//! Session tracking.
//!
//! A session groups all attack runs in a single `ai-sec` invocation.
//! Sessions can be serialised to JSON for post-analysis.

use crate::attacks::AttackResult;
use crate::scenarios::types::{PersistedScenarioEnvelope, PersistedScenarioMetaEnvelope};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const REPORT_SCHEMA_VERSION: u32 = 5;

/// Scenario-level metadata for synthetic sensitive-data exposure runs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScenarioMetadata {
    #[serde(default)]
    pub scenario_id: Option<String>,
    #[serde(default)]
    pub scenario_name: Option<String>,
    #[serde(default)]
    pub scenario_type: Option<String>,
    #[serde(default)]
    pub scenario_version: Option<String>,
    #[serde(default)]
    pub defense_profile: Option<String>,
    #[serde(default)]
    pub context_mode: Option<String>,
    #[serde(default)]
    pub retrieval_mode: Option<String>,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub session_seed: Option<String>,
    #[serde(default)]
    pub session_seed_status: Option<String>,
    #[serde(default)]
    pub active_schema_fields: Vec<String>,
    #[serde(default)]
    pub report_only_schema_fields: Vec<String>,
    #[serde(default)]
    pub sensitive_assets_count: usize,
    #[serde(default)]
    pub canary_count: usize,
    #[serde(default)]
    pub real_envelopes: Vec<PersistedScenarioEnvelope>,
    #[serde(default)]
    pub meta_envelopes: Vec<PersistedScenarioMetaEnvelope>,
    #[serde(default)]
    pub leaked_canaries: Vec<String>,
    #[serde(default)]
    pub leaked_pii_fields: Vec<String>,
    #[serde(default)]
    pub leaked_secret_types: Vec<String>,
    #[serde(default)]
    pub leaked_documents: Vec<String>,
    #[serde(default)]
    pub exposure_score: u32,
}

/// Summary statistics for a single attack category run.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttackRun {
    pub attack_id: String,
    pub attack_name: String,
    pub payloads_tested: usize,
    pub refused_count: usize,
    pub success_count: usize,
    pub partial_count: usize,
    pub inconclusive_count: usize,
    /// L0 payloads where model answered correctly and should not count as bypasses.
    pub informational_count: usize,
    /// L1 payloads are review-only and excluded from bypass scoring.
    #[serde(default)]
    pub review_only_count: usize,
    /// Number of payloads that contribute to bypass scoring.
    #[serde(default)]
    pub scoreable_payloads: usize,
    /// Cached bypass rate for diff/benchmark consumers.
    #[serde(default)]
    pub bypass_rate_pct: f32,
    /// Number of payloads generated dynamically from curated seeds.
    #[serde(default)]
    pub generated_payloads: usize,
    pub duration_ms: u64,
    pub results: Vec<AttackResult>,
}

impl AttackRun {
    /// Recompute derived metrics from the current counters.
    pub fn refresh_metrics(&mut self) {
        for result in &mut self.results {
            result.refresh_evaluation_metadata();
        }
        self.payloads_tested = self.results.len();
        self.refused_count = self
            .results
            .iter()
            .filter(|result| result.evaluation.is_refused())
            .count();
        self.success_count = self
            .results
            .iter()
            .filter(|result| result.evaluation.is_success())
            .count();
        self.partial_count = self
            .results
            .iter()
            .filter(|result| {
                matches!(
                    result.evaluation,
                    crate::engine::evaluator::EvaluationResult::Partial { .. }
                )
            })
            .count();
        self.informational_count = self
            .results
            .iter()
            .filter(|result| result.evaluation.is_informational())
            .count();
        self.inconclusive_count = self
            .results
            .iter()
            .filter(|result| {
                matches!(
                    result.evaluation,
                    crate::engine::evaluator::EvaluationResult::Inconclusive
                )
            })
            .count();
        self.generated_payloads = self
            .results
            .iter()
            .filter(|result| result.generated)
            .count();
        self.scoreable_payloads = self
            .payloads_tested
            .saturating_sub(self.informational_count + self.review_only_count);
        self.bypass_rate_pct = if self.scoreable_payloads == 0 {
            0.0
        } else {
            (self.success_count as f32 / self.scoreable_payloads as f32) * 100.0
        };
    }
}

/// Top-level aggregate statistics across all attack runs in a session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionSummary {
    pub total_payloads: usize,
    pub total_refused: usize,
    pub total_success: usize,
    pub total_partial: usize,
    pub total_inconclusive: usize,
    pub total_informational: usize,
    #[serde(default)]
    pub total_review_only: usize,
    #[serde(default)]
    pub total_scoreable_payloads: usize,
    #[serde(default)]
    pub bypass_rate_pct: f32,
    #[serde(default)]
    pub total_generated_payloads: usize,
}

impl SessionSummary {
    pub fn refresh_metrics(&mut self) {
        self.total_scoreable_payloads = self
            .total_payloads
            .saturating_sub(self.total_informational + self.total_review_only);
        self.bypass_rate_pct = if self.total_scoreable_payloads == 0 {
            0.0
        } else {
            (self.total_success as f32 / self.total_scoreable_payloads as f32) * 100.0
        };
    }

    pub fn recompute_from_runs(&mut self, runs: &[AttackRun]) {
        self.total_payloads = runs.iter().map(|run| run.payloads_tested).sum();
        self.total_refused = runs.iter().map(|run| run.refused_count).sum();
        self.total_success = runs.iter().map(|run| run.success_count).sum();
        self.total_partial = runs.iter().map(|run| run.partial_count).sum();
        self.total_inconclusive = runs.iter().map(|run| run.inconclusive_count).sum();
        self.total_informational = runs.iter().map(|run| run.informational_count).sum();
        self.total_review_only = runs.iter().map(|run| run.review_only_count).sum();
        self.total_generated_payloads = runs.iter().map(|run| run.generated_payloads).sum();
        self.refresh_metrics();
    }
}

/// Snapshot of the runtime settings used for a session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionConfig {
    pub request_timeout_secs: u64,
    pub delay_between_requests_ms: u64,
    pub concurrency: usize,
    #[serde(default)]
    pub retry_max_attempts: u32,
    #[serde(default)]
    pub retry_base_delay_ms: u64,
    #[serde(default)]
    pub retry_max_delay_ms: u64,
    #[serde(default)]
    pub generated_variants_per_attack: usize,
    #[serde(default)]
    pub generator_provider: Option<String>,
    #[serde(default)]
    pub generation_time_budget_secs: u64,
    #[serde(default)]
    pub generation_strategy: Option<String>,
}

/// Provider metadata captured for reproducible reports.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderMetadata {
    pub provider_id: String,
    pub provider_name: String,
    pub requested_model: String,
}

/// Extra report metadata used for future diff and benchmark tooling.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkMetadata {
    #[serde(default)]
    pub attack_ids: Vec<String>,
    #[serde(default)]
    pub attack_count: usize,
    #[serde(default)]
    pub scoreable_payloads: usize,
    #[serde(default)]
    pub benchmark_key: String,
}

/// External target metadata captured for HTTP-target runs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TargetMetadata {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub authenticated_user: Option<String>,
    #[serde(default)]
    pub security_profile: Option<String>,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub session_persistence: Option<String>,
    #[serde(default)]
    pub requests_sent: usize,
    #[serde(default)]
    pub tool_calls_attempted: Vec<String>,
    #[serde(default)]
    pub tool_calls_allowed: Vec<String>,
    #[serde(default)]
    pub tool_calls_denied: Vec<String>,
    #[serde(default)]
    pub redactions: Vec<String>,
}

impl TargetMetadata {
    pub fn normalize(&mut self) {
        dedupe_sorted(&mut self.tool_calls_attempted);
        dedupe_sorted(&mut self.tool_calls_allowed);
        dedupe_sorted(&mut self.tool_calls_denied);
        dedupe_sorted(&mut self.redactions);
    }
}

/// A complete test session, one per `ai-sec run` invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSession {
    #[serde(default = "default_report_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub provider: ProviderMetadata,
    #[serde(default)]
    pub target: TargetMetadata,
    #[serde(default)]
    pub config: SessionConfig,
    #[serde(default)]
    pub benchmark: BenchmarkMetadata,
    #[serde(default)]
    pub scenario: ScenarioMetadata,
    pub attacks_run: Vec<AttackRun>,
    pub summary: SessionSummary,
}

impl TestSession {
    /// Create a new session with a fresh UUID.
    pub fn new(provider: ProviderMetadata, config: SessionConfig) -> Self {
        let mut session = TestSession {
            schema_version: REPORT_SCHEMA_VERSION,
            id: Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            finished_at: None,
            provider,
            target: TargetMetadata::default(),
            config,
            benchmark: BenchmarkMetadata::default(),
            scenario: ScenarioMetadata::default(),
            attacks_run: Vec::new(),
            summary: SessionSummary::default(),
        };
        session.refresh_metrics();
        session
    }

    /// Add a completed attack run and update summary statistics.
    pub fn add_run(&mut self, mut run: AttackRun) {
        run.refresh_metrics();
        self.attacks_run.push(run);
        self.refresh_metrics();
    }

    /// Mark the session as finished and record the end time.
    pub fn finish(&mut self) {
        self.finished_at = Some(Utc::now());
        self.refresh_metrics();
    }

    /// Recompute cached report metadata and derived metrics.
    pub fn refresh_metrics(&mut self) {
        self.schema_version = REPORT_SCHEMA_VERSION;
        self.target.normalize();
        for run in &mut self.attacks_run {
            run.refresh_metrics();
        }
        self.summary.recompute_from_runs(&self.attacks_run);
        self.benchmark.attack_ids = self
            .attacks_run
            .iter()
            .map(|run| run.attack_id.clone())
            .collect();
        self.benchmark.attack_count = self.attacks_run.len();
        self.benchmark.scoreable_payloads = self.summary.total_scoreable_payloads;
        self.benchmark.benchmark_key = format!(
            "{}:{}:{}:{}",
            self.provider.provider_id,
            self.provider.requested_model,
            self.benchmark.attack_count,
            self.summary.total_scoreable_payloads
        );
        let mut leaked_canaries = std::collections::BTreeSet::new();
        let mut leaked_pii_fields = std::collections::BTreeSet::new();
        let mut leaked_secret_types = std::collections::BTreeSet::new();
        let mut leaked_documents = std::collections::BTreeSet::new();
        let mut exposure_score = 0_u32;
        for run in &self.attacks_run {
            for result in &run.results {
                leaked_canaries.extend(result.evidence.canaries.iter().cloned());
                leaked_pii_fields.extend(result.evidence.sensitive_fields.iter().cloned());
                leaked_secret_types.extend(result.evidence.secret_patterns.iter().cloned());
                leaked_documents.extend(result.evidence.documents.iter().cloned());
                exposure_score = exposure_score.saturating_add(result.damage.score);
            }
        }
        self.scenario.leaked_canaries = leaked_canaries.into_iter().collect();
        self.scenario.leaked_pii_fields = leaked_pii_fields.into_iter().collect();
        self.scenario.leaked_secret_types = leaked_secret_types.into_iter().collect();
        self.scenario.leaked_documents = leaked_documents.into_iter().collect();
        self.scenario.exposure_score = exposure_score;
        dedupe_sorted(&mut self.scenario.active_schema_fields);
        dedupe_sorted(&mut self.scenario.report_only_schema_fields);
    }
}

fn default_report_schema_version() -> u32 {
    REPORT_SCHEMA_VERSION
}

fn dedupe_sorted(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::damage::{AttackEvidence, DamageAssessment, DamageLevel};
    use crate::engine::evaluator::EvaluationResult;
    #[test]
    fn bypass_rate_excludes_l0_and_l1_from_denominator() {
        let mut run = AttackRun {
            attack_id: "test".to_string(),
            attack_name: "Test".to_string(),
            payloads_tested: 10,
            refused_count: 3,
            success_count: 2,
            partial_count: 3,
            inconclusive_count: 0,
            informational_count: 2,
            review_only_count: 2,
            scoreable_payloads: 0,
            bypass_rate_pct: 0.0,
            generated_payloads: 0,
            duration_ms: 5,
            results: vec![
                sample_result(EvaluationResult::Refused {
                    confidence: 0.9,
                    matched_phrases: vec!["cannot".to_string()],
                }),
                sample_result(EvaluationResult::Refused {
                    confidence: 0.9,
                    matched_phrases: vec!["cannot".to_string()],
                }),
                sample_result(EvaluationResult::Refused {
                    confidence: 0.9,
                    matched_phrases: vec!["cannot".to_string()],
                }),
                sample_result(EvaluationResult::Success {
                    confidence: 0.9,
                    matched_phrases: vec!["secret".to_string()],
                }),
                sample_result(EvaluationResult::Success {
                    confidence: 0.9,
                    matched_phrases: vec!["secret".to_string()],
                }),
                sample_result(EvaluationResult::Partial {
                    notes: "review".to_string(),
                }),
                sample_result(EvaluationResult::Partial {
                    notes: "review".to_string(),
                }),
                sample_result(EvaluationResult::Partial {
                    notes: "review".to_string(),
                }),
                sample_result(EvaluationResult::Informational),
                sample_result(EvaluationResult::Informational),
            ],
        };

        run.refresh_metrics();
        assert_eq!(run.scoreable_payloads, 6);
        assert!((run.bypass_rate_pct - 33.333336).abs() < 0.01);
    }

    #[test]
    fn summary_scoreable_payloads_excludes_l0_and_l1() {
        let mut summary = SessionSummary {
            total_payloads: 12,
            total_refused: 5,
            total_success: 3,
            total_partial: 2,
            total_inconclusive: 0,
            total_informational: 2,
            total_review_only: 1,
            total_scoreable_payloads: 0,
            bypass_rate_pct: 0.0,
            total_generated_payloads: 0,
        };

        summary.refresh_metrics();
        assert_eq!(summary.total_scoreable_payloads, 9);
    }

    #[test]
    fn attack_result_supports_metadata_for_reports() {
        let result = AttackResult {
            payload_id: "id".to_string(),
            payload_name: "name".to_string(),
            prompt_sent: "prompt".to_string(),
            response_received: "response".to_string(),
            transcript: Vec::new(),
            chain_planned_turns: 1,
            chain_executed_turns: 1,
            chain_completed: true,
            chain_abort_reason: None,
            evaluation: EvaluationResult::Inconclusive,
            latency_ms: 10,
            tokens_used: Some(15),
            model_used: Some("model".to_string()),
            generated: true,
            seed_payload_id: Some("seed".to_string()),
            confidence: 0.0,
            requires_review: false,
            rationale: String::new(),
            evidence: AttackEvidence {
                canaries: vec!["canary".to_string()],
                sensitive_fields: vec!["email".to_string()],
                documents: vec!["doc".to_string()],
                secret_patterns: vec!["api_key".to_string()],
                system_prompt_fragments: vec!["fragment".to_string()],
                evidence_slices: vec!["fragment".to_string()],
            },
            damage: DamageAssessment {
                level: DamageLevel::H3,
                score: 20,
                summary: "Высокая критичность".to_string(),
            },
        };

        assert_eq!(result.model_used.as_deref(), Some("model"));
        assert!(result.generated);
        assert_eq!(result.seed_payload_id.as_deref(), Some("seed"));
        assert_eq!(result.damage.score, 20);
        assert_eq!(result.chain_executed_turns, 1);
    }

    #[test]
    fn attack_run_refreshes_evaluation_metadata() {
        let mut run = AttackRun {
            attack_id: "test".to_string(),
            attack_name: "Test".to_string(),
            payloads_tested: 1,
            refused_count: 0,
            success_count: 0,
            partial_count: 1,
            inconclusive_count: 0,
            informational_count: 0,
            review_only_count: 0,
            scoreable_payloads: 0,
            bypass_rate_pct: 0.0,
            generated_payloads: 0,
            duration_ms: 1,
            results: vec![AttackResult {
                payload_id: "id".to_string(),
                payload_name: "name".to_string(),
                prompt_sent: "prompt".to_string(),
                response_received: "response".to_string(),
                transcript: Vec::new(),
                chain_planned_turns: 1,
                chain_executed_turns: 1,
                chain_completed: true,
                chain_abort_reason: None,
                evaluation: EvaluationResult::Partial {
                    notes: "Needs manual review".to_string(),
                },
                latency_ms: 1,
                tokens_used: None,
                model_used: None,
                generated: false,
                seed_payload_id: None,
                confidence: 0.0,
                requires_review: false,
                rationale: String::new(),
                evidence: AttackEvidence::default(),
                damage: DamageAssessment::default(),
            }],
        };

        run.refresh_metrics();
        assert!(run.results[0].requires_review);
        assert!(run.results[0].confidence > 0.0);
        assert!(run.results[0].rationale.contains("manual review"));
        assert_eq!(run.results[0].transcript.len(), 1);
    }

    #[test]
    fn session_refreshes_benchmark_metadata() {
        let mut session = TestSession::new(
            ProviderMetadata {
                provider_id: "openai".to_string(),
                provider_name: "OpenAI test".to_string(),
                requested_model: "gpt-test".to_string(),
            },
            SessionConfig::default(),
        );

        session.add_run(AttackRun {
            attack_id: "jailbreaking".to_string(),
            attack_name: "Jailbreaking".to_string(),
            payloads_tested: 3,
            refused_count: 1,
            success_count: 1,
            partial_count: 1,
            inconclusive_count: 0,
            informational_count: 0,
            review_only_count: 1,
            scoreable_payloads: 0,
            bypass_rate_pct: 0.0,
            generated_payloads: 0,
            duration_ms: 1,
            results: vec![
                sample_result(EvaluationResult::Refused {
                    confidence: 0.9,
                    matched_phrases: vec!["cannot".to_string()],
                }),
                sample_result(EvaluationResult::Success {
                    confidence: 0.9,
                    matched_phrases: vec!["secret".to_string()],
                }),
                sample_result(EvaluationResult::Partial {
                    notes: "manual review".to_string(),
                }),
            ],
        });

        assert_eq!(session.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(session.benchmark.attack_ids, vec!["jailbreaking"]);
        assert_eq!(session.benchmark.attack_count, 1);
        assert_eq!(session.benchmark.scoreable_payloads, 2);
    }

    #[test]
    fn attack_run_refresh_overwrites_stale_counters_from_results() {
        let mut run = AttackRun {
            attack_id: "test".to_string(),
            attack_name: "Test".to_string(),
            payloads_tested: 99,
            refused_count: 99,
            success_count: 99,
            partial_count: 99,
            inconclusive_count: 99,
            informational_count: 99,
            review_only_count: 1,
            scoreable_payloads: 0,
            bypass_rate_pct: 0.0,
            generated_payloads: 99,
            duration_ms: 1,
            results: vec![
                AttackResult {
                    payload_id: "refused".to_string(),
                    payload_name: "refused".to_string(),
                    prompt_sent: "p".to_string(),
                    response_received: "r".to_string(),
                    transcript: Vec::new(),
                    chain_planned_turns: 1,
                    chain_executed_turns: 1,
                    chain_completed: true,
                    chain_abort_reason: None,
                    evaluation: EvaluationResult::Refused {
                        confidence: 0.9,
                        matched_phrases: vec!["cannot".to_string()],
                    },
                    latency_ms: 1,
                    tokens_used: None,
                    model_used: None,
                    generated: false,
                    seed_payload_id: None,
                    confidence: 0.0,
                    requires_review: false,
                    rationale: String::new(),
                    evidence: AttackEvidence::default(),
                    damage: DamageAssessment::default(),
                },
                AttackResult {
                    payload_id: "success".to_string(),
                    payload_name: "success".to_string(),
                    prompt_sent: "p".to_string(),
                    response_received: "r".to_string(),
                    transcript: Vec::new(),
                    chain_planned_turns: 1,
                    chain_executed_turns: 1,
                    chain_completed: true,
                    chain_abort_reason: None,
                    evaluation: EvaluationResult::Success {
                        confidence: 0.9,
                        matched_phrases: vec!["secret".to_string()],
                    },
                    latency_ms: 1,
                    tokens_used: None,
                    model_used: None,
                    generated: true,
                    seed_payload_id: Some("seed".to_string()),
                    confidence: 0.0,
                    requires_review: false,
                    rationale: String::new(),
                    evidence: AttackEvidence::default(),
                    damage: DamageAssessment::default(),
                },
            ],
        };

        run.refresh_metrics();

        assert_eq!(run.payloads_tested, 2);
        assert_eq!(run.refused_count, 1);
        assert_eq!(run.success_count, 1);
        assert_eq!(run.partial_count, 0);
        assert_eq!(run.inconclusive_count, 0);
        assert_eq!(run.generated_payloads, 1);
    }

    #[test]
    fn session_summary_recomputes_from_runs_instead_of_incremental_state() {
        let mut session = TestSession::new(
            ProviderMetadata {
                provider_id: "test".to_string(),
                provider_name: "Test".to_string(),
                requested_model: "model".to_string(),
            },
            SessionConfig::default(),
        );
        session.summary.total_payloads = 999;
        session.summary.total_success = 999;
        session.attacks_run = vec![AttackRun {
            attack_id: "alpha".to_string(),
            attack_name: "Alpha".to_string(),
            payloads_tested: 0,
            refused_count: 0,
            success_count: 0,
            partial_count: 0,
            inconclusive_count: 0,
            informational_count: 0,
            review_only_count: 0,
            scoreable_payloads: 0,
            bypass_rate_pct: 0.0,
            generated_payloads: 0,
            duration_ms: 1,
            results: vec![AttackResult {
                payload_id: "id".to_string(),
                payload_name: "name".to_string(),
                prompt_sent: "prompt".to_string(),
                response_received: "response".to_string(),
                transcript: Vec::new(),
                chain_planned_turns: 1,
                chain_executed_turns: 1,
                chain_completed: true,
                chain_abort_reason: None,
                evaluation: EvaluationResult::Informational,
                latency_ms: 1,
                tokens_used: None,
                model_used: None,
                generated: false,
                seed_payload_id: None,
                confidence: 0.0,
                requires_review: false,
                rationale: String::new(),
                evidence: AttackEvidence::default(),
                damage: DamageAssessment::default(),
            }],
        }];

        session.refresh_metrics();

        assert_eq!(session.summary.total_payloads, 1);
        assert_eq!(session.summary.total_success, 0);
        assert_eq!(session.summary.total_informational, 1);
    }

    fn sample_result(evaluation: EvaluationResult) -> AttackResult {
        AttackResult {
            payload_id: "id".to_string(),
            payload_name: "name".to_string(),
            prompt_sent: "prompt".to_string(),
            response_received: "response".to_string(),
            transcript: Vec::new(),
            chain_planned_turns: 1,
            chain_executed_turns: 1,
            chain_completed: true,
            chain_abort_reason: None,
            evaluation,
            latency_ms: 1,
            tokens_used: None,
            model_used: None,
            generated: false,
            seed_payload_id: None,
            confidence: 0.0,
            requires_review: false,
            rationale: String::new(),
            evidence: AttackEvidence::default(),
            damage: DamageAssessment::default(),
        }
    }
}
