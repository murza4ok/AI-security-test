//! JSON report exporter.
//!
//! Serialises a TestSession to a JSON file for offline analysis.

use crate::engine::session::TestSession;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Write session results to a JSON file at the given path.
pub fn write_json_report(session: &TestSession, path: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    let json = serde_json::to_string_pretty(session)
        .context("Failed to serialise session to JSON")?;

    std::fs::write(path, json)
        .with_context(|| format!("Failed to write report to: {}", path.display()))?;

    Ok(())
}

/// Build a default output path in the ./results/ directory.
/// Format: results/YYYY-MM-DD_HH-MM-SS_<provider>.json
pub fn default_output_path(provider_id: &str) -> PathBuf {
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    PathBuf::from(format!("results/{}_{}.json", timestamp, provider_id))
}

/// Load all JSON session files from the results/ directory, sorted by modification time.
pub fn load_all_results() -> Result<Vec<crate::engine::session::TestSession>> {
    let mut sessions = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir("results")
        .context("Cannot read results/ directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        .collect();

    // Sort oldest → newest by modification time
    entries.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    for entry in entries {
        match load_json_report(&entry.path()) {
            Ok(s) => sessions.push(s),
            Err(e) => eprintln!("  Skipping {}: {}", entry.path().display(), e),
        }
    }

    Ok(sessions)
}

/// Load a previously saved session from a JSON file.
pub fn load_json_report(path: &Path) -> Result<crate::engine::session::TestSession> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read report: {}", path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON report: {}", path.display()))
}
