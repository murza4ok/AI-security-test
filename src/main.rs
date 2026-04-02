//! ai-sec — Educational LLM Security Testing Tool
//!
//! Entry point. Initialises logging, loads config, and dispatches to either:
//! - Interactive mode (no subcommand given)
//! - Command mode (subcommand present — scriptable/CI-friendly)

mod attacks;
mod cli;
mod config;
mod education;
mod engine;
mod payloads;
mod providers;
mod reporting;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{
    args::{Cli, Commands},
    display,
};
use engine::runner::AttackRunner;
use owo_colors::OwoColorize;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Attempt to load .env file; not a hard failure if it doesn't exist
    let _ = dotenvy::dotenv();

    // Initialise structured logging (respects RUST_LOG env var)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    // Load application config from environment variables
    let app_config = config::AppConfig::from_env()
        .context("Failed to load configuration from environment")?;

    // Destructure cli before matching on command to avoid partial moves
    let provider_override = cli.provider.clone();
    let verbose = cli.verbose;
    match cli.command {
        None => run_interactive(cli, app_config).await,
        Some(cmd) => {
            // Build a minimal Cli for the command path (only needs provider + verbose)
            let cmd_cli = cli::args::Cli {
                provider: provider_override,
                verbose,
                command: None,
            };
            run_command(cmd, cmd_cli, app_config).await
        }
    }
}

/// Interactive mode: show banners and menu-driven UI.
async fn run_interactive(cli: Cli, app_config: config::AppConfig) -> Result<()> {
    display::print_banner();
    display::print_disclaimer();
    display::print_usage_hint();

    let provider = build_provider(&cli.provider, &app_config)?;
    println!(
        "  Provider: {} {}",
        provider.name().bold(),
        "✓".green()
    );

    let loader = payloads::loader::PayloadLoader::new("payloads");

    loop {
        let selection = cli::menu::show_main_menu()?;
        match selection {
            0 => {
                // Run all attacks
                if let Some(s) = run_attacks_and_display(
                    attacks::registry::all_attacks(),
                    provider.as_ref(),
                    &loader,
                    &app_config,
                    None,
                )
                .await?
                {
                    let path = reporting::json_report::default_output_path();
                    reporting::json_report::write_json_report(&s, &path)?;
                    println!("  Report saved to: {}", path.display().to_string().green());
                }
            }
            1 => {
                // Select attack categories via checkbox menu
                let selected_ids = cli::menu::select_attack_categories()?;
                if selected_ids.is_empty() {
                    println!("  No categories selected.");
                    continue;
                }
                let selected_attacks: Vec<Arc<dyn attacks::Attack>> = selected_ids
                    .iter()
                    .filter_map(|id| attacks::registry::find_attack(id))
                    .collect();
                if let Some(s) = run_attacks_and_display(
                    selected_attacks,
                    provider.as_ref(),
                    &loader,
                    &app_config,
                    None,
                )
                .await?
                {
                    let path = reporting::json_report::default_output_path();
                    reporting::json_report::write_json_report(&s, &path)?;
                    println!("  Report saved to: {}", path.display().to_string().green());
                }
            }
            2 => {
                println!();
                println!("  Edit your {} file to change provider settings.", ".env".cyan());
                println!("  See {} for all available options.", ".env.example".cyan());
            }
            3 => {
                println!("  No saved session available yet. Run an attack first.");
            }
            4 => {
                // Educational mode
                education::list_explainable_topics();
                let all = attacks::registry::all_attacks();
                let items: Vec<String> = all
                    .iter()
                    .map(|a| format!("{} — {}", a.id(), a.name()))
                    .collect();
                if let Ok(idx) = dialoguer::Select::new()
                    .with_prompt("Choose topic to learn about")
                    .items(&items)
                    .interact()
                {
                    education::explain_attack(all[idx].id());
                }
            }
            _ => {
                println!("  Goodbye!");
                break;
            }
        }
    }

    Ok(())
}

/// Command mode: parse subcommand and execute directly.
async fn run_command(cmd: Commands, cli: Cli, app_config: config::AppConfig) -> Result<()> {
    match cmd {
        Commands::Check => {
            display::print_banner();
            println!("{}", "  Checking provider connectivity...".bold());
            println!();
            let provider = build_provider(&cli.provider, &app_config)?;
            print!("  {} ... ", provider.name().bold());
            match provider.health_check().await {
                Ok(()) => println!("{}", "✓ Connected".green().bold()),
                Err(e) => println!("{} — {}", "✗ Failed".red().bold(), e),
            }
        }

        Commands::List => {
            display::print_banner();
            let loader = payloads::loader::PayloadLoader::new("payloads");
            println!("{}", "  Available attack categories:".bold());
            println!();
            for attack in attacks::registry::all_attacks() {
                let count = attack
                    .load_payloads(&loader)
                    .map(|p| p.len())
                    .unwrap_or(0);
                println!(
                    "  {:25} {:3} payloads — {}",
                    attack.id().cyan(),
                    count,
                    attack.description()
                );
            }
            println!();
        }

        Commands::Explain { attack } => {
            display::print_banner();
            if !education::explain_attack(&attack) {
                println!("  Unknown attack ID: {}", attack.red());
                println!("  Use 'ai-sec list' to see available IDs.");
            }
        }

        Commands::Run {
            attack,
            model: _,
            output,
            limit,
        } => {
            display::print_banner();
            display::print_disclaimer();

            let provider = build_provider(&cli.provider, &app_config)?;
            println!(
                "  Provider: {} {}\n",
                provider.name().bold(),
                "✓".green()
            );

            let loader = payloads::loader::PayloadLoader::new("payloads");

            // Resolve attack IDs to implementations
            let selected: Vec<Arc<dyn attacks::Attack>> = attack
                .iter()
                .filter_map(|id| {
                    let found = attacks::registry::find_attack(id);
                    if found.is_none() {
                        eprintln!(
                            "  Warning: unknown attack ID '{}' — skipping",
                            id.yellow()
                        );
                    }
                    found
                })
                .collect();

            if selected.is_empty() {
                anyhow::bail!(
                    "No valid attack IDs. Use 'ai-sec list' to see available options."
                );
            }

            let session =
                run_attacks_and_display(selected, provider.as_ref(), &loader, &app_config, limit)
                    .await?;

            // Save JSON report: use --output path if given, otherwise auto-generate
            if let Some(s) = session {
                let path = output.unwrap_or_else(reporting::json_report::default_output_path);
                reporting::json_report::write_json_report(&s, &path)?;
                println!(
                    "  Report saved to: {}",
                    path.display().to_string().green()
                );
            }
        }
    }

    Ok(())
}

/// Execute attacks, print live progress, print summary table, return session.
async fn run_attacks_and_display(
    selected: Vec<Arc<dyn attacks::Attack>>,
    provider: &dyn providers::LLMProvider,
    loader: &payloads::loader::PayloadLoader,
    app_config: &config::AppConfig,
    limit: Option<usize>,
) -> Result<Option<engine::session::TestSession>> {
    if selected.is_empty() {
        println!("  No attacks to run.");
        return Ok(None);
    }

    let runner = AttackRunner::new(app_config.request.delay_between_requests);

    // Callback called after each individual payload result
    let current_category = std::sync::Mutex::new(String::new());

    let on_result = |attack_id: &str, result: &attacks::AttackResult| {
        // Print category header when attack group changes
        {
            let mut current = current_category.lock().unwrap();
            if current.as_str() != attack_id {
                *current = attack_id.to_string();
                let name = attacks::registry::find_attack(attack_id)
                    .map(|a| a.name().to_string())
                    .unwrap_or_else(|| attack_id.to_string());
                display::print_section(&name);
            }
        }

        // Print result label
        match result.evaluation.label() {
            "REFUSED" => display::print_refused(&result.payload_name),
            "BYPASS" => display::print_success(&result.payload_name),
            "PARTIAL" => display::print_partial(&result.payload_name),
            _ => display::print_error(&result.payload_name),
        }

        // Print response preview (~150 chars)
        if !result.response_received.is_empty() {
            let preview = display::truncate(&result.response_received, 150);
            println!("    {}", preview.dimmed());
        }
    };

    let mut attack_config = attacks::AttackConfig::default();
    attack_config.max_payloads = limit;
    attack_config.request_config = providers::RequestConfig {
        temperature: 0.7,
        max_tokens: 1024,
    };

    let session = runner
        .run_session(&selected, provider, loader, &attack_config, on_result)
        .await?;

    reporting::terminal_report::print_session_summary(&session);
    Ok(Some(session))
}

/// Build a provider from CLI override or first configured provider in app config.
fn build_provider(
    override_id: &Option<String>,
    config: &config::AppConfig,
) -> Result<Arc<dyn providers::LLMProvider>> {
    let requested = override_id.as_deref();

    // Explicit provider override
    if let Some(id) = requested {
        return match id {
            "openai" => config
                .openai
                .as_ref()
                .map(|c| {
                    Arc::new(providers::openai::OpenAIProvider::from_config(c))
                        as Arc<dyn providers::LLMProvider>
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("OpenAI not configured (missing OPENAI_API_KEY in .env)")
                }),
            "anthropic" => config
                .anthropic
                .as_ref()
                .map(|c| {
                    Arc::new(providers::anthropic::AnthropicProvider::from_config(c))
                        as Arc<dyn providers::LLMProvider>
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Anthropic not configured (missing ANTHROPIC_API_KEY in .env)"
                    )
                }),
            "ollama" => config
                .ollama
                .as_ref()
                .map(|c| {
                    Arc::new(providers::ollama::OllamaProvider::from_config(c))
                        as Arc<dyn providers::LLMProvider>
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Ollama not configured (missing OLLAMA_MODEL in .env)"
                    )
                }),
            "deepseek" => config
                .deepseek
                .as_ref()
                .map(|c| {
                    Arc::new(providers::deepseek::DeepSeekProvider::from_config(c))
                        as Arc<dyn providers::LLMProvider>
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "DeepSeek not configured (missing DEEPSEEK_API_KEY in .env)"
                    )
                }),
            "yandexgpt" => config
                .yandexgpt
                .as_ref()
                .map(|c| {
                    Arc::new(providers::yandexgpt::YandexGptProvider::from_config(c))
                        as Arc<dyn providers::LLMProvider>
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "YandexGPT not configured (missing YANDEX_API_KEY / YANDEX_FOLDER_ID in .env)"
                    )
                }),
            other => anyhow::bail!(
                "Unknown provider '{}'. Valid: openai, anthropic, ollama, deepseek, yandexgpt",
                other
            ),
        };
    }

    // Auto-select: pick the first configured provider
    if let Some(c) = &config.deepseek {
        return Ok(Arc::new(providers::deepseek::DeepSeekProvider::from_config(c)));
    }
    if let Some(c) = &config.yandexgpt {
        return Ok(Arc::new(providers::yandexgpt::YandexGptProvider::from_config(c)));
    }
    if let Some(c) = &config.anthropic {
        return Ok(Arc::new(providers::anthropic::AnthropicProvider::from_config(c)));
    }
    if let Some(c) = &config.openai {
        return Ok(Arc::new(providers::openai::OpenAIProvider::from_config(c)));
    }
    if let Some(c) = &config.ollama {
        return Ok(Arc::new(providers::ollama::OllamaProvider::from_config(c)));
    }

    anyhow::bail!(
        "No provider configured. Copy .env.example to .env and add an API key."
    )
}
