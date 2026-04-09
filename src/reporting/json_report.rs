//! JSON report exporter.
//!
//! Serialises a TestSession to a JSON file for offline analysis and migrates
//! older report versions into the current schema on read.

use crate::engine::session::{ProviderMetadata, SessionConfig, TestSession, REPORT_SCHEMA_VERSION};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SavedSessionInfo {
    pub path: PathBuf,
    pub session: TestSession,
}

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
    Ok(load_all_result_infos()?
        .into_iter()
        .map(|info| info.session)
        .collect())
}

/// Load all JSON session files from the results/ directory with their source paths.
pub fn load_all_result_infos() -> Result<Vec<SavedSessionInfo>> {
    let mut sessions = Vec::new();

    let read_dir = match std::fs::read_dir("results") {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err).context("Cannot read results/ directory"),
    };

    let mut entries: Vec<_> = read_dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        .collect();

    entries.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    entries.reverse();

    for entry in entries {
        let path = entry.path();
        match load_json_report(&path) {
            Ok(session) => sessions.push(SavedSessionInfo { path, session }),
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

    if let Some(attacks_run) = object.get_mut("attacks_run").and_then(Value::as_array_mut) {
        for run in attacks_run {
            let Some(run_object) = run.as_object_mut() else {
                continue;
            };

            if let Some(results) = run_object.get_mut("results").and_then(Value::as_array_mut) {
                for result in results {
                    let Some(result_object) = result.as_object_mut() else {
                        continue;
                    };

                    if !result_object.contains_key("evidence") {
                        let evidence = json!({
                            "canaries": result_object.remove("matched_canaries").unwrap_or_else(|| json!([])),
                            "sensitive_fields": result_object.remove("matched_sensitive_fields").unwrap_or_else(|| json!([])),
                            "documents": result_object.remove("matched_documents").unwrap_or_else(|| json!([])),
                            "secret_patterns": result_object.remove("matched_secret_patterns").unwrap_or_else(|| json!([])),
                            "system_prompt_fragments": result_object.remove("matched_system_prompt_fragments").unwrap_or_else(|| json!([])),
                        });
                        result_object.insert("evidence".to_string(), evidence);
                    }

                    if !result_object.contains_key("damage") {
                        let harm_level = result_object
                            .remove("harm_level")
                            .and_then(|value| value.as_str().map(str::to_string))
                            .unwrap_or_else(|| "l1".to_string());
                        let exposure_score = result_object
                            .remove("exposure_score")
                            .and_then(|value| value.as_u64())
                            .unwrap_or(0) as u32;
                        let (level, summary) = match harm_level.as_str() {
                            "l3" => (
                                "h3",
                                "Высокая критичность: критичный payload или прямое раскрытие секретов.",
                            ),
                            "l2" => (
                                "h2",
                                "Средняя критичность: прикладной вредоносный payload или утечка внутренних данных.",
                            ),
                            _ => (
                                "h1",
                                "Низкая критичность: разведочные или пограничные сигналы.",
                            ),
                        };
                        result_object.insert(
                            "damage".to_string(),
                            json!({
                                "level": level,
                                "score": exposure_score,
                                "summary": summary,
                            }),
                        );
                    }
                }
            }
        }
    }
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
