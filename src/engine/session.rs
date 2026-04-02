//! Session tracking.
//!
//! A session groups all attack runs in a single `ai-sec` invocation.
//! Sessions can be serialised to JSON for post-analysis.

use crate::attacks::AttackResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Summary statistics for a single attack category run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackRun {
    pub attack_id: String,
    pub attack_name: String,
    pub payloads_tested: usize,
    pub refused_count: usize,
    pub success_count: usize,
    pub partial_count: usize,
    pub inconclusive_count: usize,
    /// L0 payloads where model answered correctly — not counted as bypass
    pub informational_count: usize,
    pub duration_ms: u64,
    pub results: Vec<AttackResult>,
}

impl AttackRun {
    /// Compute bypass rate (successes / scoreable total) as a percentage.
    /// Scoreable = total minus L0 informational payloads.
    #[allow(dead_code)]
    pub fn bypass_rate_pct(&self) -> f32 {
        if self.payloads_tested == 0 {
            return 0.0;
        }
        (self.success_count as f32 / self.payloads_tested as f32) * 100.0
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
}

/// A complete test session — one per `ai-sec run` invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSession {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub provider_name: String,
    pub attacks_run: Vec<AttackRun>,
    pub summary: SessionSummary,
}

impl TestSession {
    /// Create a new session with a fresh UUID.
    pub fn new(provider_name: String) -> Self {
        TestSession {
            id: Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            finished_at: None,
            provider_name,
            attacks_run: Vec::new(),
            summary: SessionSummary::default(),
        }
    }

    /// Add a completed attack run and update summary statistics.
    pub fn add_run(&mut self, run: AttackRun) {
        self.summary.total_payloads += run.payloads_tested;
        self.summary.total_refused += run.refused_count;
        self.summary.total_success += run.success_count;
        self.summary.total_partial += run.partial_count;
        self.summary.total_inconclusive += run.inconclusive_count;
        self.summary.total_informational += run.informational_count;
        self.attacks_run.push(run);
    }

    /// Mark the session as finished and record the end time.
    pub fn finish(&mut self) {
        self.finished_at = Some(Utc::now());
    }
}
