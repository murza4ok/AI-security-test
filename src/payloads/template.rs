//! Payload template rendering.
//!
//! Payloads can contain `{{variable}}` placeholders that get substituted
//! at runtime. This allows reusable templates with per-run context injection.

#![allow(dead_code)]

use anyhow::Result;
use handlebars::Handlebars;
use serde_json::Value;

/// Renders payload prompt strings with variable substitution.
pub struct PayloadTemplate {
    engine: Handlebars<'static>,
}

impl PayloadTemplate {
    /// Create a new template renderer.
    pub fn new() -> Self {
        let mut engine = Handlebars::new();
        // Don't error on missing variables — leave them as empty string
        engine.set_strict_mode(false);
        PayloadTemplate { engine }
    }

    /// Substitute `{{variable}}` placeholders in `template` using `context`.
    ///
    /// `context` is a JSON object where keys match placeholder names.
    /// Example: `{"target_model": "GPT-4"}` replaces `{{target_model}}`.
    pub fn render(&self, template: &str, context: &Value) -> Result<String> {
        let rendered = self
            .engine
            .render_template(template, context)
            .map_err(|e| anyhow::anyhow!("Template render error: {}", e))?;
        Ok(rendered)
    }

    /// Build a default context with common variables pre-filled.
    /// Attack implementations can extend this with attack-specific values.
    pub fn default_context() -> Value {
        serde_json::json!({
            "date": chrono::Utc::now().format("%Y-%m-%d").to_string(),
        })
    }
}

impl Default for PayloadTemplate {
    fn default() -> Self {
        Self::new()
    }
}
