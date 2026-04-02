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
}
