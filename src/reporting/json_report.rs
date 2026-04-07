//! JSON report exporter.
//!
//! Serialises a TestSession to a JSON file for offline analysis and migrates
//! older report versions into the current schema on read.

use crate::engine::session::{ProviderMetadata, SessionConfig, TestSession, REPORT_SCHEMA_VERSION};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Write session results to a JSON file at the given path.
pub fn write_json_report(session: &TestSession, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    let json =
        serde_json::to_string_pretty(session).context("Failed to serialise session to JSON")?;

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
pub fn load_all_results() -> Result<Vec<TestSession>> {
    let mut sessions = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir("results")
        .context("Cannot read results/ directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        .collect();

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
pub fn load_json_report(path: &Path) -> Result<TestSession> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read report: {}", path.display()))?;

    let value: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON report: {}", path.display()))?;

    load_json_report_from_value(value)
        .with_context(|| format!("Failed to parse JSON report: {}", path.display()))
}

fn load_json_report_from_value(mut value: Value) -> Result<TestSession> {
    migrate_legacy_report_value(&mut value);
    let mut session: TestSession = serde_json::from_value(value)?;
    session.refresh_metrics();
    Ok(session)
}

fn migrate_legacy_report_value(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };

    object
        .entry("schema_version")
        .or_insert(json!(REPORT_SCHEMA_VERSION));

    if !object.contains_key("provider") {
        let provider_name = object
            .remove("provider_name")
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "unknown".to_string());

        object.insert(
            "provider".to_string(),
            serde_json::to_value(ProviderMetadata {
                provider_id: "unknown".to_string(),
                provider_name,
                requested_model: "unknown".to_string(),
            })
            .expect("provider metadata is serializable"),
        );
    }

    object
        .entry("config")
        .or_insert_with(|| serde_json::to_value(SessionConfig::default()).unwrap());
    object.entry("benchmark").or_insert_with(|| json!({}));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_legacy_report_shape() {
        let legacy = json!({
            "id": "session-1",
            "started_at": "2026-04-07T10:00:00Z",
            "finished_at": "2026-04-07T10:01:00Z",
            "provider_name": "DeepSeek test",
            "attacks_run": [],
            "summary": {
                "total_payloads": 3,
                "total_refused": 1,
                "total_success": 1,
                "total_partial": 1,
                "total_inconclusive": 0,
                "total_informational": 0
            }
        });

        let session = load_json_report_from_value(legacy).unwrap();
        assert_eq!(session.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(session.provider.provider_name, "DeepSeek test");
        assert_eq!(session.provider.provider_id, "unknown");
    }
}
