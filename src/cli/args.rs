//! CLI argument definitions using clap.
//!
//! Two modes:
//! - No subcommand → interactive menu
//! - Subcommand present → direct command execution (scriptable)

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// ai-sec — Educational LLM Security Testing Tool
#[derive(Parser, Debug)]
#[command(
    name = "ai-sec",
    about = "Educational CLI for testing LLM security vulnerabilities",
    long_about = "ai-sec helps security researchers understand and test LLM attack surfaces.\n\
                  For educational and authorized testing purposes only.",
    version
)]
pub struct Cli {
    /// Override the provider to use (openai, anthropic, ollama)
    #[arg(short, long, global = true, env = "AISEC_PROVIDER")]
    pub provider: Option<String>,

    /// Increase output verbosity (use -v or -vv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available subcommands for non-interactive (scripted) use
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run one or more attack categories against the configured provider
    Run {
        /// Attack category IDs to run (e.g., jailbreaking, prompt_injection)
        /// Use `ai-sec list` to see all available IDs.
        #[arg(short, long, required = true)]
        attack: Vec<String>,

        /// Override model name for this run
        #[arg(short, long)]
        model: Option<String>,

        /// Save results to a JSON file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Limit number of payloads per attack category (useful for quick tests)
        #[arg(short, long)]
        limit: Option<usize>,

        /// Generate up to N additional payload variants per attack using DeepSeek
        #[arg(long)]
        generated: Option<usize>,

        /// Application scenario for scenario-driven attacks such as sensitive_data_exposure
        #[arg(long)]
        app_scenario: Option<String>,

        /// Override the default synthetic fixture root
        #[arg(long)]
        fixture_root: Option<PathBuf>,

        /// Retrieval mode for scenario-driven attacks: full or subset
        #[arg(long)]
        retrieval_mode: Option<String>,

        /// Override the scenario manifest path
        #[arg(long)]
        scenario_config: Option<PathBuf>,

        /// Optional tenant identifier for synthetic multi-tenant scenarios
        #[arg(long)]
        tenant: Option<String>,

        /// Optional deterministic seed for scenario assembly
        #[arg(long)]
        session_seed: Option<String>,
    },

    /// List all available attack categories and their payload counts
    List,

    /// Show educational explanation of an attack category
    Explain {
        /// Attack category ID (e.g., jailbreaking, token_attacks)
        attack: String,
    },

    /// Verify connectivity and credentials for all configured providers
    Check,

    /// Display a saved JSON report in human-readable format for manual review
    Review {
        /// Path to the JSON report file (e.g. results/2026-04-02_14-30.json)
        file: std::path::PathBuf,
    },

    /// Compare results from multiple sessions side by side (one per provider)
    Compare {
        /// JSON report files to compare (e.g. results/..._deepseek.json results/..._yandexgpt.json)
        /// If omitted, auto-loads all files from the results/ directory
        #[arg(value_name = "FILE")]
        files: Vec<std::path::PathBuf>,
    },

    /// Show an overview of saved sessions in results/
    Sessions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_command_parses_model_override() {
        let cli = Cli::parse_from([
            "ai-sec",
            "run",
            "--attack",
            "jailbreaking",
            "--model",
            "gpt-4.1-mini",
        ]);

        match cli.command {
            Some(Commands::Run {
                model,
                attack,
                generated,
                app_scenario,
                ..
            }) => {
                assert_eq!(attack, vec!["jailbreaking"]);
                assert_eq!(model.as_deref(), Some("gpt-4.1-mini"));
                assert_eq!(generated, None);
                assert_eq!(app_scenario, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn run_command_parses_generated_variants() {
        let cli = Cli::parse_from([
            "ai-sec",
            "run",
            "--attack",
            "prompt_injection",
            "--generated",
            "3",
        ]);

        match cli.command {
            Some(Commands::Run {
                attack,
                generated,
                app_scenario,
                ..
            }) => {
                assert_eq!(attack, vec!["prompt_injection"]);
                assert_eq!(generated, Some(3));
                assert_eq!(app_scenario, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn run_command_parses_sensitive_data_flags() {
        let cli = Cli::parse_from([
            "ai-sec",
            "run",
            "--attack",
            "sensitive_data_exposure",
            "--app-scenario",
            "support_bot",
            "--retrieval-mode",
            "subset",
            "--tenant",
            "tenant-a",
            "--session-seed",
            "demo",
        ]);

        match cli.command {
            Some(Commands::Run {
                attack,
                app_scenario,
                retrieval_mode,
                tenant,
                session_seed,
                ..
            }) => {
                assert_eq!(attack, vec!["sensitive_data_exposure"]);
                assert_eq!(app_scenario.as_deref(), Some("support_bot"));
                assert_eq!(retrieval_mode.as_deref(), Some("subset"));
                assert_eq!(tenant.as_deref(), Some("tenant-a"));
                assert_eq!(session_seed.as_deref(), Some("demo"));
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }
}
