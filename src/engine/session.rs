//! Session tracking.
//!
//! A session groups all attack runs in a single `ai-sec` invocation.
//! Sessions can be serialised to JSON for post-analysis.

use crate::attacks::AttackResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const REPORT_SCHEMA_VERSION: u32 = 2;

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
    pub duration_ms: u64,
    pub results: Vec<AttackResult>,
}

impl AttackRun {
    /// Recompute derived metrics from the current counters.
    pub fn refresh_metrics(&mut self) {
        self.scoreable_payloads = self
            .payloads_tested
            .saturating_sub(self.informational_count + self.review_only_count);
        self.bypass_rate_pct = if self.scoreable_payloads == 0 {
            0.0
        } else {
            (self.success_count as f32 / self.scoreable_payloads as f32) * 100.0
        };
    }

    #[allow(dead_code)]
    pub fn bypass_rate_pct(&self) -> f32 {
        self.bypass_rate_pct
    }

    pub fn scoreable_payloads(&self) -> usize {
        self.scoreable_payloads
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

    pub fn scoreable_payloads(&self) -> usize {
        self.total_scoreable_payloads
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
    pub config: SessionConfig,
    #[serde(default)]
    pub benchmark: BenchmarkMetadata,
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
            config,
            benchmark: BenchmarkMetadata::default(),
            attacks_run: Vec::new(),
            summary: SessionSummary::default(),
        };
        session.refresh_metrics();
        session
    }

    /// Add a completed attack run and update summary statistics.
    pub fn add_run(&mut self, mut run: AttackRun) {
        run.refresh_metrics();
        self.summary.total_payloads += run.payloads_tested;
        self.summary.total_refused += run.refused_count;
        self.summary.total_success += run.success_count;
        self.summary.total_partial += run.partial_count;
        self.summary.total_inconclusive += run.inconclusive_count;
        self.summary.total_informational += run.informational_count;
        self.summary.total_review_only += run.review_only_count;
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
        for run in &mut self.attacks_run {
            run.refresh_metrics();
        }
        self.summary.refresh_metrics();
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
    }
}

fn default_report_schema_version() -> u32 {
    REPORT_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::evaluator::EvaluationResult;
    use crate::payloads::loader::HarmLevel;

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
            duration_ms: 5,
            results: Vec::new(),
        };

        run.refresh_metrics();
        assert_eq!(run.scoreable_payloads(), 6);
        assert!((run.bypass_rate_pct() - 33.333336).abs() < 0.01);
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
        };

        summary.refresh_metrics();
        assert_eq!(summary.scoreable_payloads(), 9);
    }

    #[test]
    fn attack_result_supports_metadata_for_reports() {
        let result = AttackResult {
            payload_id: "id".to_string(),
            payload_name: "name".to_string(),
            prompt_sent: "prompt".to_string(),
            response_received: "response".to_string(),
            harm_level: HarmLevel::L2,
            evaluation: EvaluationResult::Inconclusive,
            latency_ms: 10,
            tokens_used: Some(15),
            model_used: Some("model".to_string()),
        };

        assert_eq!(result.harm_level, HarmLevel::L2);
        assert_eq!(result.model_used.as_deref(), Some("model"));
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
            duration_ms: 1,
            results: Vec::new(),
        });

        assert_eq!(session.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(session.benchmark.attack_ids, vec!["jailbreaking"]);
        assert_eq!(session.benchmark.attack_count, 1);
        assert_eq!(session.benchmark.scoreable_payloads, 2);
    }
}
