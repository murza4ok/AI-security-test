//! ai-sec - Educational LLM Security Testing Tool
//!
//! Entry point. Initializes logging, loads config, and dispatches to either:
//! - Interactive mode (no subcommand given)
//! - Command mode (subcommand present - scriptable/CI-friendly)

mod attacks;
mod cli;
mod config;
mod education;
mod engine;
mod generator;
mod payloads;
mod providers;
mod reporting;
mod scenarios;

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
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    let app_config =
        config::AppConfig::from_env().context("Failed to load configuration from environment")?;

    let provider_override = cli.provider.clone();
    let verbose = cli.verbose;
    match cli.command {
        None => run_interactive(cli, app_config).await,
        Some(cmd) => {
            let cmd_cli = cli::args::Cli {
                provider: provider_override,
                verbose,
                command: None,
            };
            run_command(cmd, cmd_cli, app_config).await
        }
    }
}

async fn run_interactive(cli: Cli, app_config: config::AppConfig) -> Result<()> {
    display::print_banner();
    display::print_disclaimer();
    display::print_usage_hint();

    let providers = build_all_providers(&cli.provider, None, &app_config)?;
    println!(
        "  Providers: {}",
        providers
            .iter()
            .map(|provider| provider.name().bold().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let loader = payloads::loader::PayloadLoader::new("payloads");

    loop {
        match cli::menu::show_main_menu()? {
            0 => {
                run_all_providers(
                    &providers,
                    attacks::registry::all_standard_attacks(),
                    &loader,
                    &app_config,
                    None,
                    None,
                    None,
                )
                .await?;
            }
            1 => {
                let selected_ids = cli::menu::select_attack_categories()?;
                if selected_ids.is_empty() {
                    println!("  No categories selected.");
                    continue;
                }

                let selected_attacks: Vec<Arc<dyn attacks::Attack>> = selected_ids
                    .iter()
                    .filter_map(|id| attacks::registry::find_attack(id))
                    .collect();

                run_all_providers(
                    &providers,
                    selected_attacks,
                    &loader,
                    &app_config,
                    None,
                    None,
                    None,
                )
                .await?;
            }
            2 => {
                println!();
                println!(
                    "  Edit your {} file to change provider settings.",
                    ".env".cyan()
                );
                println!("  See {} for all available options.", ".env.example".cyan());
            }
            3 => show_saved_sessions_interactive()?,
            4 => {
                education::list_explainable_topics();
                let all = attacks::registry::all_attacks();
                let items: Vec<String> = all
                    .iter()
                    .map(|attack| format!("{} - {}", attack.id(), attack.name()))
                    .collect();

                if let Ok(index) = dialoguer::Select::new()
                    .with_prompt("Choose topic to learn about")
                    .items(&items)
                    .interact()
                {
                    education::explain_attack(all[index].id());
                }
            }
            _ => {
                println!("  Goodbye.");
                break;
            }
        }
    }

    Ok(())
}

async fn run_command(cmd: Commands, cli: Cli, app_config: config::AppConfig) -> Result<()> {
    match cmd {
        Commands::Review { file } => {
            let session = reporting::json_report::load_json_report(&file)?;
            reporting::terminal_report::print_session_summary(&session);
            reporting::terminal_report::print_session_review(&session);
        }
        Commands::Compare { files } => {
            display::print_banner();
            let sessions: Vec<_> = if files.is_empty() {
                reporting::json_report::load_all_results()?
            } else {
                files
                    .iter()
                    .map(|path| reporting::json_report::load_json_report(path))
                    .collect::<anyhow::Result<Vec<_>>>()?
            };

            if sessions.is_empty() {
                println!("  No session files found for comparison.");
            } else if sessions.len() == 1 {
                println!("  Only one session found. Showing single-session summary.");
                reporting::terminal_report::print_session_summary(&sessions[0]);
            } else {
                reporting::terminal_report::print_comparison_table(&sessions);
            }
        }
        Commands::Sessions => {
            display::print_banner();
            let sessions = reporting::json_report::load_all_result_infos()?;
            reporting::terminal_report::print_saved_sessions_overview(&sessions);
        }
        Commands::Check => {
            display::print_banner();
            println!("{}", "  Checking provider connectivity...".bold());
            println!();

            let providers = build_all_providers(&cli.provider, None, &app_config)?;
            for provider in &providers {
                print!("  {:15} ... ", provider.name().bold());
                match provider.health_check().await {
                    Ok(()) => println!("{}", "OK".green().bold()),
                    Err(error) => println!("{} - {}", "FAILED".red().bold(), error),
                }
            }
        }
        Commands::List => {
            display::print_banner();
            let loader = payloads::loader::PayloadLoader::new("payloads");
            println!("{}", "  Available attack categories:".bold());
            println!();
            for attack in attacks::registry::all_attacks() {
                let count = attack.load_payloads(&loader).map(|payloads| payloads.len()).unwrap_or(0);
                println!(
                    "  {:25} {:3} payloads - {}",
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
            model,
            output,
            limit,
            generated,
            app_scenario,
            fixture_root,
            retrieval_mode,
            scenario_config,
            tenant,
            session_seed,
        } => {
            display::print_banner();
            display::print_disclaimer();

            let providers = build_all_providers(&cli.provider, model.as_deref(), &app_config)?;
            let loader = payloads::loader::PayloadLoader::new("payloads");

            let selected: Vec<Arc<dyn attacks::Attack>> = attack
                .iter()
                .filter_map(|id| {
                    let found = attacks::registry::find_attack(id);
                    if found.is_none() {
                        eprintln!("  Warning: unknown attack ID '{}' - skipping", id.yellow());
                    }
                    found
                })
                .collect();

            if selected.is_empty() {
                anyhow::bail!("No valid attack IDs. Use 'ai-sec list' to see available options.");
            }

            let includes_sensitive_data_exposure = selected
                .iter()
                .any(|selected_attack| selected_attack.id() == "sensitive_data_exposure");
            if includes_sensitive_data_exposure && app_scenario.is_none() {
                anyhow::bail!(
                    "--app-scenario is required when running sensitive_data_exposure"
                );
            }

            let scenario = build_scenario_config(
                app_scenario.as_deref(),
                fixture_root.as_ref(),
                retrieval_mode.as_deref(),
                scenario_config.as_ref(),
                tenant.as_deref(),
                session_seed.as_deref(),
            )?;

            if output.is_some() && providers.len() > 1 {
                eprintln!(
                    "  {} --output is ignored when running multiple providers; reports are auto-named per provider.",
                    "Warning:".yellow()
                );
            }

            for provider in &providers {
                let session = run_attacks_and_display(
                    selected.clone(),
                    provider.as_ref(),
                    &loader,
                    &app_config,
                    limit,
                    generated,
                    scenario.clone(),
                )
                .await?;

                if let Some(session) = session {
                    let path = if providers.len() == 1 {
                        output.clone().unwrap_or_else(|| {
                            reporting::json_report::default_output_path(provider.id())
                        })
                    } else {
                        reporting::json_report::default_output_path(provider.id())
                    };
                    reporting::json_report::write_json_report(&session, &path)?;
                    println!("  Report saved to: {}", path.display().to_string().green());
                }
            }
        }
    }

    Ok(())
}

fn show_saved_sessions_interactive() -> Result<()> {
    let sessions = reporting::json_report::load_all_result_infos()?;
    if sessions.is_empty() {
        println!("  No reports found in results/. Run attacks first.");
        return Ok(());
    }

    reporting::terminal_report::print_saved_sessions_overview(&sessions);
    match cli::menu::select_saved_sessions_action()? {
        0 => {}
        1 => {
            let labels: Vec<String> = sessions
                .iter()
                .map(|saved| {
                    format!(
                        "{} | {} | {}",
                        saved.session.started_at.format("%Y-%m-%d %H:%M UTC"),
                        saved.session.provider.provider_name,
                        saved.path.display()
                    )
                })
                .collect();

            if let Some(index) = cli::menu::select_saved_session(&labels)? {
                reporting::terminal_report::print_session_summary(&sessions[index].session);
                reporting::terminal_report::print_session_review(&sessions[index].session);
            }
        }
        2 => {
            let selected: Vec<_> = sessions.iter().map(|saved| saved.session.clone()).collect();
            reporting::terminal_report::print_comparison_table(&selected);
        }
        _ => {}
    }

    Ok(())
}

async fn run_all_providers(
    providers: &[Arc<dyn providers::LLMProvider>],
    attacks: Vec<Arc<dyn attacks::Attack>>,
    loader: &payloads::loader::PayloadLoader,
    app_config: &config::AppConfig,
    limit: Option<usize>,
    generated: Option<usize>,
    scenario: Option<scenarios::types::ScenarioRunConfig>,
) -> Result<()> {
    for provider in providers {
        let session = run_attacks_and_display(
            attacks.clone(),
            provider.as_ref(),
            loader,
            app_config,
            limit,
            generated,
            scenario.clone(),
        )
        .await?;

        if let Some(session) = session {
            let path = reporting::json_report::default_output_path(provider.id());
            reporting::json_report::write_json_report(&session, &path)?;
            println!("  Report saved to: {}", path.display().to_string().green());
        }
    }
    Ok(())
}

async fn run_attacks_and_display(
    selected: Vec<Arc<dyn attacks::Attack>>,
    provider: &dyn providers::LLMProvider,
    loader: &payloads::loader::PayloadLoader,
    app_config: &config::AppConfig,
    limit: Option<usize>,
    generated: Option<usize>,
    scenario: Option<scenarios::types::ScenarioRunConfig>,
) -> Result<Option<engine::session::TestSession>> {
    if selected.is_empty() {
        println!("  No attacks to run.");
        return Ok(None);
    }

    println!();
    println!(
        "{}",
        format!("  ╔══ {} ════════════════════════════════════════", provider.name())
            .cyan()
            .bold()
    );
    println!();

    let runner = AttackRunner::new(app_config.request.delay_between_requests);
    let current_category = std::sync::Mutex::new(String::new());

    let on_result = |attack_id: &str, result: &attacks::AttackResult| {
        let mut current = current_category.lock().unwrap();
        if current.as_str() != attack_id {
            *current = attack_id.to_string();
            let name = attacks::registry::find_attack(attack_id)
                .map(|attack| attack.name().to_string())
                .unwrap_or_else(|| attack_id.to_string());
            display::print_section(&name);
        }
        drop(current);

        match result.evaluation.label() {
            "REFUSED" => display::print_refused(&result.payload_name),
            "BYPASS" => display::print_success(&result.payload_name),
            "PARTIAL" => display::print_partial(&result.payload_name),
            "INFO" => display::print_informational(&result.payload_name),
            "INCONCLUSIVE" => {
                if result.response_received.starts_with("ERROR:") {
                    display::print_error(&result.payload_name)
                } else {
                    display::print_partial(&result.payload_name)
                }
            }
            _ => display::print_error(&result.payload_name),
        }

        if !result.response_received.is_empty() {
            let preview = display::truncate(&result.response_received, 150);
            println!("    {}", preview.dimmed());
        }
    };

    let mut attack_config = attacks::AttackConfig::default();
    attack_config.max_payloads = limit;
    attack_config.concurrency = app_config.request.concurrency;
    attack_config.request_config = providers::RequestConfig {
        temperature: 0.7,
        max_tokens: 1024,
    };

    if let Some(variants_per_attack) = generated.filter(|count| *count > 0) {
        attack_config.generation = Some(generator::GenerationConfig::with_defaults(
            variants_per_attack,
        ));
        attack_config.generator_provider = Some(
            build_generation_provider(app_config, None).context(
                "Generated mode requires DeepSeek to be configured as the generator provider",
            )?,
        );
    }
    attack_config.scenario = scenario.clone();

    let session = runner
        .run_session(
            &selected,
            provider,
            loader,
            &attack_config,
            engine::session::SessionConfig {
                request_timeout_secs: app_config.request.timeout.as_secs(),
                delay_between_requests_ms: app_config.request.delay_between_requests.as_millis()
                    as u64,
                concurrency: app_config.request.concurrency,
                retry_max_attempts: app_config.request.retry_max_attempts,
                retry_base_delay_ms: app_config.request.retry_base_delay.as_millis() as u64,
                retry_max_delay_ms: app_config.request.retry_max_delay.as_millis() as u64,
                generated_variants_per_attack: generated.unwrap_or(0),
                generator_provider: attack_config
                    .generator_provider
                    .as_ref()
                    .map(|provider| provider.name().to_string()),
                generation_time_budget_secs: attack_config
                    .generation
                    .as_ref()
                    .map(|config| config.time_budget.as_secs())
                    .unwrap_or(0),
                generation_strategy: attack_config
                    .generation
                    .as_ref()
                    .map(|config| format!("{:?}", config.strategy).to_lowercase()),
            },
            on_result,
        )
        .await?;

    if let Some(scenario) = scenario {
        let definition = scenarios::loader::load_scenario(&scenario)?;
        let mut session = session;
        session.scenario.scenario_id = Some(definition.manifest.id.clone());
        session.scenario.scenario_name = Some(definition.manifest.name.clone());
        session.scenario.scenario_type = Some(definition.manifest.scenario_type.clone());
        session.scenario.sensitive_assets_count =
            definition.hidden_assets.len() + definition.retrieval_assets.len();
        session.scenario.canary_count = definition.canaries.len();
        reporting::terminal_report::print_session_summary(&session);
        return Ok(Some(session));
    }

    reporting::terminal_report::print_session_summary(&session);
    Ok(Some(session))
}

fn build_generation_provider(
    config: &config::AppConfig,
    model_override: Option<&str>,
) -> Result<Arc<dyn providers::LLMProvider>> {
    build_provider_by_id("deepseek", model_override, config)
}

fn build_scenario_config(
    app_scenario: Option<&str>,
    fixture_root: Option<&std::path::PathBuf>,
    retrieval_mode: Option<&str>,
    scenario_config: Option<&std::path::PathBuf>,
    tenant: Option<&str>,
    session_seed: Option<&str>,
) -> Result<Option<scenarios::types::ScenarioRunConfig>> {
    let Some(scenario_id) = app_scenario else {
        return Ok(None);
    };

    let retrieval_mode = retrieval_mode
        .map(|value| {
            scenarios::types::RetrievalMode::parse(value)
                .ok_or_else(|| anyhow::anyhow!("Invalid retrieval mode '{}'. Use full or subset", value))
        })
        .transpose()?
        .unwrap_or_else(|| {
            if scenario_id == "internal_rag_bot" {
                scenarios::types::RetrievalMode::Subset
            } else {
                scenarios::types::RetrievalMode::Full
            }
        });

    Ok(Some(scenarios::types::ScenarioRunConfig {
        scenario_id: scenario_id.to_string(),
        fixture_root: fixture_root
            .cloned()
            .unwrap_or_else(|| std::path::PathBuf::from("fixtures/sensitive_data_exposure")),
        retrieval_mode,
        scenario_config_path: scenario_config.cloned(),
        tenant: tenant.map(str::to_string),
        session_seed: session_seed.map(str::to_string),
    }))
}

fn build_all_providers(
    override_id: &Option<String>,
    model_override: Option<&str>,
    config: &config::AppConfig,
) -> Result<Vec<Arc<dyn providers::LLMProvider>>> {
    if let Some(id) = override_id.as_deref() {
        return build_provider_by_id(id, model_override, config).map(|provider| vec![provider]);
    }

    let mut list: Vec<Arc<dyn providers::LLMProvider>> = Vec::new();
    let timeout = config.request.timeout;
    let retry_settings = providers::RetrySettings {
        max_attempts: config.request.retry_max_attempts,
        base_delay: config.request.retry_base_delay,
        max_delay: config.request.retry_max_delay,
    };

    if let Some(provider_config) = &config.deepseek {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(providers::deepseek::DeepSeekProvider::from_config(
            &provider_config,
            timeout,
            retry_settings,
        )));
    }
    if let Some(provider_config) = &config.yandexgpt {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(
            providers::yandexgpt::YandexGptProvider::from_config(
                &provider_config,
                timeout,
                retry_settings,
            ),
        ));
    }
    if let Some(provider_config) = &config.anthropic {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(
            providers::anthropic::AnthropicProvider::from_config(
                &provider_config,
                timeout,
                retry_settings,
            ),
        ));
    }
    if let Some(provider_config) = &config.openai {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(providers::openai::OpenAIProvider::from_config(
            &provider_config,
            timeout,
            retry_settings,
        )));
    }
    if let Some(provider_config) = &config.ollama {
        let mut provider_config = provider_config.clone();
        if let Some(model) = model_override {
            provider_config.model = model.to_string();
        }
        list.push(Arc::new(providers::ollama::OllamaProvider::from_config(
            &provider_config,
            timeout,
            retry_settings,
        )));
    }

    if list.is_empty() {
        anyhow::bail!("No provider configured. Copy .env.example to .env and add an API key.");
    }

    Ok(list)
}

fn build_provider_by_id(
    id: &str,
    model_override: Option<&str>,
    config: &config::AppConfig,
) -> Result<Arc<dyn providers::LLMProvider>> {
    let timeout = config.request.timeout;
    let retry_settings = providers::RetrySettings {
        max_attempts: config.request.retry_max_attempts,
        base_delay: config.request.retry_base_delay,
        max_delay: config.request.retry_max_delay,
    };

    match id {
        "openai" => config
            .openai
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::openai::OpenAIProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            })
            .ok_or_else(|| anyhow::anyhow!("OpenAI not configured (missing OPENAI_API_KEY in .env)")),
        "anthropic" => config
            .anthropic
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::anthropic::AnthropicProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Anthropic not configured (missing ANTHROPIC_API_KEY in .env)")
            }),
        "ollama" => config
            .ollama
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::ollama::OllamaProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            })
            .ok_or_else(|| anyhow::anyhow!("Ollama not configured (missing OLLAMA_MODEL in .env)")),
        "deepseek" => config
            .deepseek
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::deepseek::DeepSeekProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
            })
            .ok_or_else(|| anyhow::anyhow!("DeepSeek not configured (missing DEEPSEEK_API_KEY in .env)")),
        "yandexgpt" => config
            .yandexgpt
            .as_ref()
            .map(|provider_config| {
                let mut provider_config = provider_config.clone();
                if let Some(model) = model_override {
                    provider_config.model = model.to_string();
                }
                Arc::new(providers::yandexgpt::YandexGptProvider::from_config(
                    &provider_config,
                    timeout,
                    retry_settings,
                )) as Arc<dyn providers::LLMProvider>
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
    }
}
