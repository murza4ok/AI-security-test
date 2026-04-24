use crate::scenarios::types::{
    RetrievalMode, ScenarioAsset, ScenarioDefinition, ScenarioRunConfig,
};

pub fn select_documents(
    definition: &ScenarioDefinition,
    config: &ScenarioRunConfig,
    query: &str,
) -> Vec<ScenarioAsset> {
    if !definition.manifest.context.retrieval_enabled {
        return Vec::new();
    }

    match config.retrieval_mode {
        RetrievalMode::Full => definition.retrieval_assets.clone(),
        RetrievalMode::Subset => select_subset(definition, config, query),
    }
}

fn select_subset(
    definition: &ScenarioDefinition,
    config: &ScenarioRunConfig,
    query: &str,
) -> Vec<ScenarioAsset> {
    let keywords: Vec<String> = query
        .to_lowercase()
        .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .filter(|part| part.len() > 2)
        .map(|part| part.to_string())
        .collect();

    let mut scored: Vec<(i32, u64, ScenarioAsset)> = definition
        .retrieval_assets
        .iter()
        .cloned()
        .map(|asset| {
            let haystack = format!(
                "{} {}",
                asset.source.to_lowercase(),
                asset.content.to_lowercase()
            );
            let score = keywords
                .iter()
                .filter(|keyword| haystack.contains(keyword.as_str()))
                .count() as i32;
            let seed_rank = stable_seed_rank(
                config.session_seed.as_deref().unwrap_or("seedless"),
                query,
                &asset.source,
            );
            (score, seed_rank, asset)
        })
        .collect();

    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.source.cmp(&right.2.source))
    });

    let top_n = definition.manifest.context.top_n.unwrap_or(2);
    scored
        .into_iter()
        .take(top_n.max(1))
        .map(|(_, _, asset)| asset)
        .collect()
}

fn stable_seed_rank(seed: &str, query: &str, source: &str) -> u64 {
    stable_hash(&format!(
        "{}::{}::{}",
        seed,
        query.to_lowercase(),
        source.to_lowercase()
    ))
}

fn stable_hash(input: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
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
            loaded_definition: None,
        };
        let definition = load_scenario(&config).unwrap();
        let first_docs: Vec<String> =
            select_documents(&definition, &config, "admin endpoint vendor secret")
                .into_iter()
                .map(|asset| asset.source)
                .collect();
        let second_docs: Vec<String> =
            select_documents(&definition, &config, "admin endpoint vendor secret")
                .into_iter()
                .map(|asset| asset.source)
                .collect();

        assert!(!first_docs.is_empty());
        assert_eq!(first_docs, second_docs);
    }

    #[test]
    fn subset_retrieval_uses_session_seed_for_tie_breaks() {
        let first = ScenarioRunConfig {
            scenario_id: "internal_rag_bot".to_string(),
            fixture_root: PathBuf::from("fixtures/sensitive_data_exposure"),
            retrieval_mode: RetrievalMode::Subset,
            scenario_config_path: None,
            tenant: None,
            session_seed: Some("alpha".to_string()),
            loaded_definition: None,
        };
        let second = ScenarioRunConfig {
            session_seed: Some("omega".to_string()),
            ..first.clone()
        };
        let definition = load_scenario(&first).unwrap();

        let first_docs: Vec<String> = select_documents(&definition, &first, "overview")
            .into_iter()
            .map(|asset| asset.source)
            .collect();
        let second_docs: Vec<String> = select_documents(&definition, &second, "overview")
            .into_iter()
            .map(|asset| asset.source)
            .collect();

        assert_ne!(first_docs, second_docs);
    }
}
