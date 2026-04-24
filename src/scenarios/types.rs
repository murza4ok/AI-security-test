use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioManifest {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub scenario_type: String,
    #[serde(default = "default_scenario_version")]
    pub version: String,
    #[serde(default)]
    pub defense_profile: Option<String>,
    pub context: ScenarioContextConfig,
    pub sensitivity: SensitivityManifest,
    #[serde(default)]
    pub threat_model: ThreatModelManifest,
}

fn default_scenario_version() -> String {
    "1.0".to_string()
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
    #[serde(default = "default_prompt_placement")]
    pub prompt_placement: String,
    #[serde(default = "default_hidden_context_policy")]
    pub hidden_context_policy: String,
    #[serde(default)]
    pub mask_pii: bool,
    #[serde(default = "default_include_secret_store")]
    pub include_secret_store: bool,
}

fn default_prompt_placement() -> String {
    "system".to_string()
}

fn default_hidden_context_policy() -> String {
    "raw".to_string()
}

fn default_include_secret_store() -> bool {
    true
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreatModelManifest {
    #[serde(default)]
    pub protected_assets: Vec<String>,
    #[serde(default)]
    pub attacker_capabilities: Vec<String>,
    #[serde(default)]
    pub trust_boundaries: Vec<String>,
    #[serde(default)]
    pub expected_safe_behavior: Vec<String>,
    #[serde(default)]
    pub expected_failure_modes: Vec<String>,
    #[serde(default)]
    pub severity_mapping: Vec<String>,
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
    #[serde(skip)]
    pub loaded_definition: Option<Arc<ScenarioDefinition>>,
}

#[derive(Debug, Clone)]
pub struct ScenarioEnvelope {
    pub system_prompt: String,
    pub retrieved_documents: Vec<ScenarioAsset>,
    pub user_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedScenarioEnvelope {
    pub payload_id: String,
    pub payload_name: String,
    pub payload_prompt: String,
    pub system_prompt: String,
    pub user_prompt: String,
    #[serde(default)]
    pub retrieved_documents: Vec<ScenarioAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedScenarioMetaEnvelope {
    pub payload_id: String,
    pub payload_name: String,
    pub scenario_id: String,
    pub scenario_name: String,
    pub context_mode: String,
    pub retrieval_mode: String,
    pub retrieval_enabled: bool,
    pub top_n: Option<usize>,
    pub memory_enabled: bool,
    pub prompt_placement: String,
    pub hidden_context_policy: String,
    pub mask_pii: bool,
    pub include_secret_store: bool,
    #[serde(default)]
    pub defense_profile: Option<String>,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub session_seed: Option<String>,
    #[serde(default)]
    pub session_seed_applied: bool,
    #[serde(default)]
    pub hidden_asset_sources: Vec<String>,
    #[serde(default)]
    pub retrieved_document_sources: Vec<String>,
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
