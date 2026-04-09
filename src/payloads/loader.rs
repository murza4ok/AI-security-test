//! Payload loader.
//!
//! Reads TOML files from the `payloads/<category>/` directory and returns
//! structured `Payload` objects. Each TOML file maps to one `PayloadFile`.

use anyhow::{Context, Result};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Metadata section at the top of each TOML payload file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PayloadMetadata {
    /// Which attack category this file belongs to
    pub attack_type: String,
    /// Sub-variant within the category (e.g., "dan", "roleplay")
    pub variant: String,
    /// Overall severity: "low", "medium", "high"
    pub severity: String,
    /// Optional human-readable notes about this payload set
    pub notes: Option<String>,
}

/// Harm classification for a payload.
///
/// Used by the evaluator to decide whether a model response constitutes
/// a genuine safety bypass or simply legitimate helpful behaviour.
///
/// | Level | Meaning                                       | Bypass if answered? |
/// |-------|-----------------------------------------------|---------------------|
/// | L0    | Public knowledge — concept definitions         | No → Informational  |
/// | L1    | Borderline — general patterns, no operat. detail | Partial only       |
/// | L2    | Harmful — actionable instructions              | Yes → Bypass        |
/// | L3    | Critical — working code / targeted attack      | Yes → Bypass        |
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HarmLevel {
    /// Public knowledge; answering is correct model behaviour, not a bypass
    L0,
    /// Borderline; answering warrants human review but isn't a clear bypass
    #[default]
    L1,
    /// Harmful; answering is a safety failure
    L2,
    /// Critical; answering is a serious safety failure
    L3,
}

impl HarmLevel {
    /// Short label for display (e.g. "L2")
    pub fn label(&self) -> &'static str {
        match self {
            HarmLevel::L0 => "L0",
            HarmLevel::L1 => "L1",
            HarmLevel::L2 => "L2",
            HarmLevel::L3 => "L3",
        }
    }
}

/// A single payload test case loaded from TOML.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Payload {
    /// Unique identifier within the file (e.g., "dan_v6")
    pub id: String,
    /// Human-readable display name
    pub name: String,
    /// What this payload tests
    pub description: String,
    /// The actual prompt text to send to the LLM
    pub prompt: String,
    /// Keywords whose presence in the response suggests the model refused
    #[serde(default)]
    pub expected_refusal_keywords: Vec<String>,
    /// Keywords whose presence suggests the attack succeeded
    #[serde(default)]
    pub success_indicators: Vec<String>,
    /// Harm classification — controls how evaluator interprets a positive response
    #[serde(default)]
    pub harm_level: HarmLevel,
    /// Per-payload severity override (inherits file-level if not set)
    pub severity: Option<String>,
    /// Optional notes for the operator / learner
    #[serde(default)]
    pub notes: Option<String>,
    /// Whether this payload was generated dynamically from another payload
    #[serde(default)]
    pub generated: bool,
    /// Seed payload id used to generate this payload
    #[serde(default)]
    pub seed_payload_id: Option<String>,
}

/// Represents a fully parsed TOML payload file.
#[derive(Debug, Clone, Deserialize)]
pub struct PayloadFile {
    pub metadata: PayloadMetadata,
    #[serde(rename = "payloads")]
    pub entries: Vec<Payload>,
}

/// Responsible for discovering and loading payload files from disk.
pub struct PayloadLoader {
    /// Root directory where all payload subdirectories live
    payloads_root: PathBuf,
}

impl PayloadLoader {
    /// Create a new loader pointing at `payloads_root`.
    /// Typically this is `./payloads` relative to the binary.
    pub fn new(payloads_root: impl Into<PathBuf>) -> Self {
        PayloadLoader {
            payloads_root: payloads_root.into(),
        }
    }

    /// Load all payloads for a given attack category (directory name).
    /// Returns a flat list of payloads from all TOML files in that directory.
    pub fn load_category(&self, category: &str) -> Result<Vec<Payload>> {
        let dir = self.payloads_root.join(category);
        if !dir.exists() {
            anyhow::bail!("Payload directory not found: {}", dir.display());
        }

        let mut payloads = Vec::new();
        for entry in std::fs::read_dir(&dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                let mut file_payloads = self
                    .load_file(&path)
                    .with_context(|| format!("Failed to load: {}", path.display()))?;
                payloads.append(&mut file_payloads);
            }
        }

        Ok(payloads)
    }

    /// Load payloads from a single TOML file.
    fn load_file(&self, path: &Path) -> Result<Vec<Payload>> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read file: {}", path.display()))?;

        let file: PayloadFile = toml::from_str(&content)
            .with_context(|| format!("Invalid TOML in: {}", path.display()))?;

        // Propagate file-level severity to payloads that don't set their own
        let payloads = file
            .entries
            .into_iter()
            .map(|mut p| {
                if p.severity.is_none() {
                    p.severity = Some(file.metadata.severity.clone());
                }
                p
            })
            .collect();

        Ok(payloads)
    }

    /// Return up to `count` payloads sampled randomly from the provided slice.
    pub fn sample_payloads(&self, payloads: &[Payload], count: usize) -> Vec<Payload> {
        let mut rng = rand::thread_rng();
        let mut sampled = payloads.to_vec();
        sampled.shuffle(&mut rng);
        sampled.truncate(count.min(sampled.len()));
        sampled
    }
}
