use crate::app::{providers, runtime};
use crate::attacks;
use crate::cli::{self, display};
use crate::config;
use crate::education;
use crate::payloads;
use crate::providers::LLMProvider;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::sync::Arc;

pub async fn run_interactive(
    cli_args: cli::args::Cli,
    app_config: config::AppConfig,
) -> Result<()> {
    display::print_banner();
    display::print_disclaimer();
    display::print_usage_hint();
    let providers_available = print_provider_availability(&cli_args, &app_config);

    let loader = payloads::loader::PayloadLoader::new("payloads");

    loop {
        match cli::menu::show_main_menu(providers_available)? {
            cli::menu::MainMenuAction::RunAllAttacks => {
                let providers_list =
                    match providers::build_all_providers(&cli_args.provider, None, &app_config) {
                        Ok(providers_list) => providers_list,
                        Err(error) => {
                            print_attack_run_blocked(&error.to_string());
                            continue;
                        }
                    };
                print_selected_providers(&providers_list);
                runtime::run_all_providers(
                    &providers_list,
                    attacks::registry::all_attacks(),
                    &loader,
                    &app_config,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            }
            cli::menu::MainMenuAction::RunSelectedAttacks => {
                let selected_ids = cli::menu::select_attack_categories()?;
                if selected_ids.is_empty() {
                    println!("  No categories selected.");
                    continue;
                }

                let providers_list =
                    match providers::build_all_providers(&cli_args.provider, None, &app_config) {
                        Ok(providers_list) => providers_list,
                        Err(error) => {
                            print_attack_run_blocked(&error.to_string());
                            continue;
                        }
                    };
                let selected_attacks: Vec<Arc<dyn attacks::Attack>> = selected_ids
                    .iter()
                    .filter_map(|id| attacks::registry::find_attack(id))
                    .collect();

                print_selected_providers(&providers_list);
                runtime::run_all_providers(
                    &providers_list,
                    selected_attacks,
                    &loader,
                    &app_config,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            }
            cli::menu::MainMenuAction::ProviderSetupHint => {
                println!();
                println!(
                    "  Edit your {} file to change provider settings.",
                    ".env".cyan()
                );
                println!("  See {} for all available options.", ".env.example".cyan());
                if let Some(provider_override) = cli_args.provider.as_deref() {
                    println!("  Current CLI override: {}", provider_override.bold());
                }
                println!("  This menu does not edit provider settings automatically.");
            }
            cli::menu::MainMenuAction::BrowseSavedSessions => {
                runtime::show_saved_sessions_interactive()?
            }
            cli::menu::MainMenuAction::LearnAttackFamilies => {
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
            cli::menu::MainMenuAction::Quit => {
                println!("  Goodbye.");
                break;
            }
        }
    }

    Ok(())
}

fn print_provider_availability(cli_args: &cli::args::Cli, app_config: &config::AppConfig) -> bool {
    match providers::build_all_providers(&cli_args.provider, None, app_config) {
        Ok(providers_list) => {
            print_selected_providers(&providers_list);
            true
        }
        Err(error) => {
            println!("  Providers: {}", "not configured".yellow().bold());
            println!(
                "  Attack runs are unavailable: {}",
                error.to_string().yellow()
            );
            println!("  Attack-run entries are hidden until a provider is configured.");
            println!("  You can still browse saved sessions and read attack explainers.");
            false
        }
    }
}

fn print_selected_providers(providers_list: &[Arc<dyn LLMProvider>]) {
    println!(
        "  Providers: {}",
        providers_list
            .iter()
            .map(|provider| provider.name().bold().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

fn print_attack_run_blocked(error: &str) {
    println!();
    println!("  {} {}", "Cannot start attack run:".yellow().bold(), error);
    println!("  Use the provider setup hint or pass a valid --provider override.");
}
