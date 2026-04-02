//! Payload loader.
//!
//! Reads TOML files from the `payloads/<category>/` directory and returns
//! structured `Payload` objects. Each TOML file maps to one `PayloadFile`.

#![allow(dead_code)]

use anyhow::{Context, Result};
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
    /// Per-payload severity override (inherits file-level if not set)
    pub severity: Option<String>,
    /// Optional notes for the operator / learner
    pub notes: Option<String>,
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

    /// Count how many payloads exist for each category.
    /// Returns a sorted list of (category_name, count) pairs.
    pub fn list_categories(&self) -> Result<Vec<(String, usize)>> {
        let mut results = Vec::new();
        let entries = std::fs::read_dir(&self.payloads_root)
            .with_context(|| "Cannot read payloads root directory")?;

        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                let count = self.load_category(&name).map(|p| p.len()).unwrap_or(0);
                results.push((name, count));
            }
        }

        results.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(results)
    }
}
