//! Interactive menu system.
//!
//! Provides a dialoguer-based menu for the interactive (no-argument) mode.
//! Guides the user through provider selection, attack selection, and results.

#![allow(dead_code)]

use crate::attacks::registry::all_attacks;
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};

/// Top-level menu choices
const MENU_ITEMS: &[&str] = &[
    "Run All Attacks",
    "Select Attack Categories",
    "Configure Provider (edit .env)",
    "View Last Session Report",
    "Educational Mode — Learn About Attacks",
    "Quit",
];

/// Present the main menu and return the selected action index.
pub fn show_main_menu() -> Result<usize> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Main Menu")
        .items(MENU_ITEMS)
        .default(0)
        .interact()?;
    Ok(selection)
}

/// Present attack category selection with checkboxes.
/// Returns a list of selected attack IDs.
pub fn select_attack_categories() -> Result<Vec<String>> {
    let attacks = all_attacks();
    let items: Vec<String> = attacks
        .iter()
        .map(|a| format!("{:30} — {}", a.name(), a.description()))
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select attack categories (Space to toggle, Enter to confirm)")
        .items(&items)
        .interact()?;

    Ok(selections
        .into_iter()
        .map(|i| attacks[i].id().to_string())
        .collect())
}

/// Ask the user to choose a provider from those configured in the environment.
/// Returns the provider ID string.
pub fn select_provider(available: &[String]) -> Result<String> {
    if available.is_empty() {
        anyhow::bail!(
            "No providers configured. Copy .env.example to .env and add your API key."
        );
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select provider")
        .items(available)
        .default(0)
        .interact()?;

    Ok(available[selection].clone())
}

/// Simple yes/no prompt. Returns true for yes.
pub fn confirm(prompt: &str) -> Result<bool> {
    let items = &["Yes", "No"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()?;
    Ok(selection == 0)
}
