use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioManifest {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub scenario_type: String,
    pub context: ScenarioContextConfig,
    pub sensitivity: SensitivityManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioContextConfig {
    pub mode: String,
    #[serde(default)]
    pub retrieval_enabled: bool,
    #[serde(default)]
    pub top_n: Option<usize>,
    #[serde(default)]
    pub memory_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityManifest {
    #[serde(default)]
    pub canary_files: Vec<String>,
    #[serde(default)]
    pub pii_fields: Vec<String>,
    #[serde(default)]
    pub credential_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryDefinition {
    pub id: String,
    pub value: String,
    pub secret_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSecretFile {
    pub canaries: Vec<CanaryDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveValue {
    pub field: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioAsset {
    pub id: String,
    pub source: String,
    pub kind: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ScenarioDefinition {
    pub manifest: ScenarioManifest,
    pub system_prompt: String,
    pub hidden_assets: Vec<ScenarioAsset>,
    pub retrieval_assets: Vec<ScenarioAsset>,
    pub canaries: Vec<CanaryDefinition>,
    pub sensitive_values: Vec<SensitiveValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrievalMode {
    Full,
    Subset,
}

impl RetrievalMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "full" => Some(Self::Full),
            "subset" => Some(Self::Subset),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioRunConfig {
    pub scenario_id: String,
    pub fixture_root: PathBuf,
    pub retrieval_mode: RetrievalMode,
    pub scenario_config_path: Option<PathBuf>,
    pub tenant: Option<String>,
    pub session_seed: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScenarioEnvelope {
    pub system_prompt: String,
    #[allow(dead_code)]
    pub hidden_context: String,
    pub retrieved_documents: Vec<ScenarioAsset>,
    pub user_prompt: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExposureSignals {
    #[serde(default)]
    pub matched_canaries: Vec<String>,
    #[serde(default)]
    pub matched_sensitive_fields: Vec<String>,
    #[serde(default)]
    pub matched_documents: Vec<String>,
    #[serde(default)]
    pub matched_secret_patterns: Vec<String>,
    #[serde(default)]
    pub matched_system_prompt_fragments: Vec<String>,
    #[serde(default)]
    pub exposure_score: u32,
}
