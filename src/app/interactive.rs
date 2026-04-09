use crate::app::{providers, runtime};
use crate::attacks;
use crate::cli::{self, display};
use crate::config;
use crate::education;
use crate::payloads;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::sync::Arc;

pub async fn run_interactive(cli_args: cli::args::Cli, app_config: config::AppConfig) -> Result<()> {
    display::print_banner();
    display::print_disclaimer();
    display::print_usage_hint();

    let providers_list = providers::build_all_providers(&cli_args.provider, None, &app_config)?;
    println!(
        "  Providers: {}",
        providers_list
            .iter()
            .map(|provider| provider.name().bold().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let loader = payloads::loader::PayloadLoader::new("payloads");

    loop {
        match cli::menu::show_main_menu()? {
            0 => {
                runtime::run_all_providers(
                    &providers_list,
                    attacks::registry::all_attacks(),
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

                runtime::run_all_providers(
                    &providers_list,
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
            3 => runtime::show_saved_sessions_interactive()?,
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
