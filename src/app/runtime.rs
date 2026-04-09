use crate::app::{providers, scenarios};
use crate::attacks;
use crate::cli::display;
use crate::config;
use crate::engine::{self, runner::AttackRunner};
use crate::generator;
use crate::payloads;
use crate::providers::LLMProvider;
use crate::reporting;
use crate::scenarios as app_scenarios;
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::sync::Arc;

pub fn show_saved_sessions_interactive() -> Result<()> {
    let sessions = reporting::json_report::load_all_result_infos()?;
    if sessions.is_empty() {
        println!("  No reports found in results/. Run attacks first.");
        return Ok(());
    }

    reporting::terminal_report::print_saved_sessions_overview(&sessions);
    match crate::cli::menu::select_saved_sessions_action()? {
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

            if let Some(index) = crate::cli::menu::select_saved_session(&labels)? {
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

pub async fn run_all_providers(
    providers_list: &[Arc<dyn LLMProvider>],
    attacks_list: Vec<Arc<dyn attacks::Attack>>,
    loader: &payloads::loader::PayloadLoader,
    app_config: &config::AppConfig,
    limit: Option<usize>,
    generated: Option<usize>,
    scenario: Option<app_scenarios::types::ScenarioRunConfig>,
) -> Result<()> {
    for provider in providers_list {
        let session = run_attacks_and_display(
            attacks_list.clone(),
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

pub async fn run_attacks_and_display(
    selected: Vec<Arc<dyn attacks::Attack>>,
    provider: &dyn LLMProvider,
    loader: &payloads::loader::PayloadLoader,
    app_config: &config::AppConfig,
    limit: Option<usize>,
    generated: Option<usize>,
    scenario: Option<app_scenarios::types::ScenarioRunConfig>,
) -> Result<Option<engine::session::TestSession>> {
    if selected.is_empty() {
        println!("  No attacks to run.");
        return Ok(None);
    }

    println!();
    println!("{}", format!("== {} ==", provider.name()).cyan().bold());
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
    attack_config.request_config = crate::providers::RequestConfig {
        temperature: 0.7,
        max_tokens: 1024,
    };

    if let Some(variants_per_attack) = generated.filter(|count| *count > 0) {
        attack_config.generation = Some(generator::GenerationConfig::with_defaults(
            variants_per_attack,
        ));
        attack_config.generator_provider = Some(
            providers::build_generation_provider(app_config, None).context(
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
        let definition = app_scenarios::loader::load_scenario(&scenario)?;
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

pub async fn run_command(
    cmd: crate::cli::args::Commands,
    cli: crate::cli::args::Cli,
    app_config: config::AppConfig,
) -> Result<()> {
    match cmd {
        crate::cli::args::Commands::Review { file } => {
            let session = reporting::json_report::load_json_report(&file)?;
            reporting::terminal_report::print_session_summary(&session);
            reporting::terminal_report::print_session_review(&session);
        }
        crate::cli::args::Commands::Compare { files } => {
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
        crate::cli::args::Commands::Sessions => {
            display::print_banner();
            let sessions = reporting::json_report::load_all_result_infos()?;
            reporting::terminal_report::print_saved_sessions_overview(&sessions);
        }
        crate::cli::args::Commands::Check => {
            display::print_banner();
            println!("{}", "  Checking provider connectivity...".bold());
            println!();

            let providers_list = providers::build_all_providers(&cli.provider, None, &app_config)?;
            for provider in &providers_list {
                print!("  {:15} ... ", provider.name().bold());
                match provider.health_check().await {
                    Ok(()) => println!("{}", "OK".green().bold()),
                    Err(error) => println!("{} - {}", "FAILED".red().bold(), error),
                }
            }
        }
        crate::cli::args::Commands::List => {
            display::print_banner();
            let loader = payloads::loader::PayloadLoader::new("payloads");
            println!("{}", "  Available attack categories:".bold());
            println!();
            for attack in attacks::registry::all_attacks() {
                let count = attack
                    .load_payloads(&loader)
                    .map(|payloads| payloads.len())
                    .unwrap_or(0);
                println!(
                    "  {:25} {:3} payloads - {}",
                    attack.id().cyan(),
                    count,
                    attack.description()
                );
            }
            println!();
        }
        crate::cli::args::Commands::Explain { attack } => {
            display::print_banner();
            if !crate::education::explain_attack(&attack) {
                println!("  Unknown attack ID: {}", attack.red());
                println!("  Use 'ai-sec list' to see available IDs.");
            }
        }
        crate::cli::args::Commands::Run {
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

            let providers_list =
                providers::build_all_providers(&cli.provider, model.as_deref(), &app_config)?;
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

            let scenario = scenarios::build_scenario_config(
                app_scenario.as_deref(),
                fixture_root.as_ref(),
                retrieval_mode.as_deref(),
                scenario_config.as_ref(),
                tenant.as_deref(),
                session_seed.as_deref(),
            )?;

            if output.is_some() && providers_list.len() > 1 {
                eprintln!(
                    "  {} --output is ignored when running multiple providers; reports are auto-named per provider.",
                    "Warning:".yellow()
                );
            }

            for provider in &providers_list {
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
                    let path = if providers_list.len() == 1 {
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
