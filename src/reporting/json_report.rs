//! JSON report exporter.
//!
//! Serialises a TestSession to a JSON file for offline analysis.

use crate::engine::session::TestSession;
use anyhow::{Context, Result};
use std::path::Path;

/// Write session results to a JSON file at the given path.
pub fn write_json_report(session: &TestSession, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(session)
        .context("Failed to serialise session to JSON")?;

    std::fs::write(path, json)
        .with_context(|| format!("Failed to write report to: {}", path.display()))?;

    Ok(())
}
