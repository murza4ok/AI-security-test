use crate::scenarios::types::{RetrievalMode, ScenarioAsset, ScenarioDefinition, ScenarioRunConfig};

pub fn select_documents(definition: &ScenarioDefinition, config: &ScenarioRunConfig, query: &str) -> Vec<ScenarioAsset> {
    match config.retrieval_mode {
        RetrievalMode::Full => definition.retrieval_assets.clone(),
        RetrievalMode::Subset => select_subset(definition, query),
    }
}

fn select_subset(definition: &ScenarioDefinition, query: &str) -> Vec<ScenarioAsset> {
    let keywords: Vec<String> = query
        .to_lowercase()
        .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .filter(|part| part.len() > 2)
        .map(|part| part.to_string())
        .collect();

    let mut scored: Vec<(i32, ScenarioAsset)> = definition
        .retrieval_assets
        .iter()
        .cloned()
        .map(|asset| {
            let haystack = format!("{} {}", asset.source.to_lowercase(), asset.content.to_lowercase());
            let score = keywords
                .iter()
                .filter(|keyword| haystack.contains(keyword.as_str()))
                .count() as i32;
            (score, asset)
        })
        .collect();

    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.source.cmp(&right.1.source))
    });

    let top_n = definition.manifest.context.top_n.unwrap_or(2);
    scored
        .into_iter()
        .take(top_n.max(1))
        .map(|(_, asset)| asset)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenarios::{loader::load_scenario, types::ScenarioRunConfig};
    use std::path::PathBuf;

    #[test]
    fn subset_retrieval_is_deterministic() {
        let config = ScenarioRunConfig {
            scenario_id: "internal_rag_bot".to_string(),
            fixture_root: PathBuf::from("fixtures/sensitive_data_exposure"),
            retrieval_mode: RetrievalMode::Subset,
            scenario_config_path: None,
            tenant: None,
            session_seed: Some("demo".to_string()),
        };
        let definition = load_scenario(&config).unwrap();
        let docs = select_documents(&definition, &config, "admin endpoint vendor secret");

        assert!(!docs.is_empty());
        assert_eq!(docs[0].source, "infra_access.md");
    }
}
