//! Standardized damage and evidence model.

use serde::{Deserialize, Serialize};

/// Estimated damage severity for a successful or partial attack outcome.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DamageLevel {
    /// Low impact: reconnaissance, schema discovery, weak hints.
    #[default]
    H1,
    /// Medium impact: internal data or multi-field PII leakage.
    H2,
    /// High impact: credential disclosure, raw confidential text, direct exfiltration.
    H3,
}

impl DamageLevel {
    pub fn criticality(&self) -> &'static str {
        match self {
            DamageLevel::H1 => {
                "Низкая критичность: разведка, перечисление полей или слабые признаки утечки."
            }
            DamageLevel::H2 => {
                "Средняя критичность: утечка внутренних данных, PII или служебного контента."
            }
            DamageLevel::H3 => {
                "Высокая критичность: утечка секретов, credential-like значений или сырого конфиденциального текста."
            }
        }
    }
}

/// Structured evidence attached to an attack result.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttackEvidence {
    #[serde(default)]
    pub canaries: Vec<String>,
    #[serde(default)]
    pub sensitive_fields: Vec<String>,
    #[serde(default)]
    pub documents: Vec<String>,
    #[serde(default)]
    pub secret_patterns: Vec<String>,
    #[serde(default)]
    pub system_prompt_fragments: Vec<String>,
}

impl AttackEvidence {
    pub fn is_empty(&self) -> bool {
        self.canaries.is_empty()
            && self.sensitive_fields.is_empty()
            && self.documents.is_empty()
            && self.secret_patterns.is_empty()
            && self.system_prompt_fragments.is_empty()
    }
}

/// Standardized damage assessment for one attack result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageAssessment {
    #[serde(default)]
    pub level: DamageLevel,
    #[serde(default)]
    pub score: u32,
    #[serde(default)]
    pub summary: String,
}

impl Default for DamageAssessment {
    fn default() -> Self {
        Self {
            level: DamageLevel::H1,
            score: 0,
            summary: "Признаки ущерба не обнаружены".to_string(),
        }
    }
}
