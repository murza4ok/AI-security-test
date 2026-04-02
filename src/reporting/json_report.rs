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
/// Format: results/YYYY-MM-DD_HH-MM-SS.json
pub fn default_output_path() -> PathBuf {
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    PathBuf::from(format!("results/{}.json", timestamp))
}

/// Load a previously saved session from a JSON file.
pub fn load_json_report(path: &Path) -> Result<crate::engine::session::TestSession> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read report: {}", path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON report: {}", path.display()))
}
