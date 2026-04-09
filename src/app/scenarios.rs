use crate::scenarios::types::{RetrievalMode, ScenarioRunConfig};
use anyhow::Result;
use std::path::PathBuf;

pub fn build_scenario_config(
    app_scenario: Option<&str>,
    fixture_root: Option<&PathBuf>,
    retrieval_mode: Option<&str>,
    scenario_config: Option<&PathBuf>,
    tenant: Option<&str>,
    session_seed: Option<&str>,
) -> Result<Option<ScenarioRunConfig>> {
    let Some(scenario_id) = app_scenario else {
        return Ok(None);
    };

    let retrieval_mode = retrieval_mode
        .map(|value| {
            RetrievalMode::parse(value)
                .ok_or_else(|| anyhow::anyhow!("Invalid retrieval mode '{}'. Use full or subset", value))
        })
        .transpose()?
        .unwrap_or_else(|| {
            if scenario_id == "internal_rag_bot" {
                RetrievalMode::Subset
            } else {
                RetrievalMode::Full
            }
        });

    Ok(Some(ScenarioRunConfig {
        scenario_id: scenario_id.to_string(),
        fixture_root: fixture_root
            .cloned()
            .unwrap_or_else(|| PathBuf::from("fixtures/sensitive_data_exposure")),
        retrieval_mode,
        scenario_config_path: scenario_config.cloned(),
        tenant: tenant.map(str::to_string),
        session_seed: session_seed.map(str::to_string),
    }))
}
