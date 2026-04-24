use crate::scenarios::types::{
    CanaryDefinition, ScenarioAsset, ScenarioDefinition, ScenarioManifest, ScenarioRunConfig,
    ScenarioSecretFile, SensitiveValue,
};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn load_scenario(config: &ScenarioRunConfig) -> Result<ScenarioDefinition> {
    let scenario_root = config.fixture_root.join(&config.scenario_id);
    let manifest_path = config
        .scenario_config_path
        .clone()
        .unwrap_or_else(|| scenario_root.join("scenario.toml"));
    let manifest: ScenarioManifest = toml::from_str(
        &std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path.display()))?,
    )
    .with_context(|| format!("Invalid scenario manifest {}", manifest_path.display()))?;

    let system_prompt_path = scenario_root.join("system_prompt.txt");
    let system_prompt = std::fs::read_to_string(&system_prompt_path)
        .with_context(|| format!("Failed to read {}", system_prompt_path.display()))?;

    let mut hidden_assets = Vec::new();
    let mut retrieval_assets = Vec::new();
    let mut sensitive_values = Vec::new();

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&scenario_root)
        .with_context(|| format!("Failed to read {}", scenario_root.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();

    for path in entries {
        if path.is_dir() {
            if path.file_name().and_then(|v| v.to_str()) == Some("kb") {
                let kb_assets = load_directory_assets(&path, "retrieved_document")?;
                for asset in &kb_assets {
                    sensitive_values.extend(extract_sensitive_values(
                        asset,
                        &manifest.sensitivity.pii_fields,
                    ));
                }
                retrieval_assets.extend(kb_assets);
            }
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if matches!(
            file_name,
            "scenario.toml" | "system_prompt.txt" | "secrets.toml"
        ) {
            continue;
        }

        let kind = match path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
        {
            "csv" => "table",
            "json" => "record_set",
            "md" => "note",
            _ => "asset",
        };
        let asset = load_asset(&path, kind)?;
        sensitive_values.extend(extract_sensitive_values(
            &asset,
            &manifest.sensitivity.pii_fields,
        ));
        hidden_assets.push(asset);
    }

    let canaries = load_canaries(&scenario_root, &manifest.sensitivity.canary_files)?;
    for canary in &canaries {
        sensitive_values.push(SensitiveValue {
            field: canary.secret_type.clone(),
            value: canary.value.clone(),
        });
    }

    hidden_assets.push(ScenarioAsset {
        id: "secrets".to_string(),
        source: "secrets.toml".to_string(),
        kind: "secret_store".to_string(),
        content: render_canaries(&canaries),
    });

    Ok(ScenarioDefinition {
        manifest,
        system_prompt,
        hidden_assets,
        retrieval_assets,
        canaries,
        sensitive_values,
    })
}

pub fn resolve_scenario_definition(config: &ScenarioRunConfig) -> Result<Arc<ScenarioDefinition>> {
    if let Some(definition) = &config.loaded_definition {
        return Ok(Arc::clone(definition));
    }

    Ok(Arc::new(load_scenario(config)?))
}

fn load_canaries(root: &Path, files: &[String]) -> Result<Vec<CanaryDefinition>> {
    let mut canaries = Vec::new();
    for file in files {
        let path = root.join(file);
        let parsed: ScenarioSecretFile = toml::from_str(
            &std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?,
        )
        .with_context(|| format!("Invalid secrets file {}", path.display()))?;
        canaries.extend(parsed.canaries);
    }
    Ok(canaries)
}

fn load_directory_assets(dir: &Path, kind: &str) -> Result<Vec<ScenarioAsset>> {
    let mut assets = Vec::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .collect();
    entries.sort();
    for path in entries {
        assets.push(load_asset(&path, kind)?);
    }
    Ok(assets)
}

fn load_asset(path: &Path, kind: &str) -> Result<ScenarioAsset> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    Ok(ScenarioAsset {
        id: path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("asset")
            .to_string(),
        source: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("asset")
            .to_string(),
        kind: kind.to_string(),
        content,
    })
}

fn render_canaries(canaries: &[CanaryDefinition]) -> String {
    canaries
        .iter()
        .map(|canary| format!("{} = {} ({})", canary.id, canary.value, canary.secret_type))
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_sensitive_values(asset: &ScenarioAsset, pii_fields: &[String]) -> Vec<SensitiveValue> {
    match asset.source.rsplit('.').next().unwrap_or_default() {
        "csv" => extract_from_csv(&asset.content, pii_fields),
        "json" => extract_from_json(&asset.content, pii_fields),
        _ => Vec::new(),
    }
}

fn extract_from_csv(content: &str, pii_fields: &[String]) -> Vec<SensitiveValue> {
    let mut values = Vec::new();
    let mut lines = content.lines();
    let Some(header_line) = lines.next() else {
        return values;
    };
    let headers: Vec<String> = header_line
        .split(',')
        .map(|header| header.trim().to_string())
        .collect();
    for line in lines {
        let columns: Vec<&str> = line.split(',').map(str::trim).collect();
        for (index, header) in headers.iter().enumerate() {
            if pii_fields.iter().any(|field| field == header) {
                if let Some(value) = columns.get(index) {
                    values.push(SensitiveValue {
                        field: header.clone(),
                        value: (*value).to_string(),
                    });
                }
            }
        }
    }
    values
}

fn extract_from_json(content: &str, pii_fields: &[String]) -> Vec<SensitiveValue> {
    let Ok(value) = serde_json::from_str::<Value>(content) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    collect_json_sensitive_values(&value, pii_fields, &mut values);
    values
}

fn collect_json_sensitive_values(
    value: &Value,
    pii_fields: &[String],
    values: &mut Vec<SensitiveValue>,
) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if pii_fields.iter().any(|field| field == key) {
                    if let Some(text) = nested.as_str() {
                        values.push(SensitiveValue {
                            field: key.clone(),
                            value: text.to_string(),
                        });
                    }
                }
                collect_json_sensitive_values(nested, pii_fields, values);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_json_sensitive_values(item, pii_fields, values);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenarios::types::RetrievalMode;

    #[test]
    fn loads_support_bot_fixture() {
        let definition = load_scenario(&ScenarioRunConfig {
            scenario_id: "support_bot".to_string(),
            fixture_root: PathBuf::from("fixtures/sensitive_data_exposure"),
            retrieval_mode: RetrievalMode::Full,
            scenario_config_path: None,
            tenant: None,
            session_seed: Some("demo".to_string()),
            loaded_definition: None,
        })
        .unwrap();

        assert_eq!(definition.manifest.id, "support_bot");
        assert!(!definition.canaries.is_empty());
        assert!(!definition.hidden_assets.is_empty());
    }
}
