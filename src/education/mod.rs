//! Educational content module.
//!
//! Provides the `explain` command functionality: displays detailed
//! explanations and resource links for each attack category.

use crate::attacks::registry::all_attacks;
use owo_colors::OwoColorize;

/// Print the educational explainer for the given attack ID.
/// Returns false if the attack ID is not found.
pub fn explain_attack(attack_id: &str) -> bool {
    let attacks = all_attacks();
    let Some(attack) = attacks.iter().find(|a| a.id() == attack_id) else {
        return false;
    };

    println!();
    println!(
        "{}",
        format!(
            "╔══ EDUCATIONAL MODE: {} ══",
            attack.name()
        )
        .cyan()
        .bold()
    );
    println!();

    // Print the explainer text (pre-formatted multi-line string)
    for line in attack.educational_explainer().lines() {
        if line.starts_with(|c: char| c.is_uppercase()) && !line.starts_with("  ") {
            // Section headers in the explainer — make them bold
            println!("  {}", line.bold().bright_blue());
        } else {
            println!("  {}", line);
        }
    }

    // Print resource links
    let resources = attack.resources();
    if !resources.is_empty() {
        println!();
        println!("  {}", "── FURTHER READING ──────────────────────────".bright_blue().bold());
        for (i, res) in resources.iter().enumerate() {
            println!("  [{}] {} — {}", i + 1, res.title.bold(), res.source.dimmed());
            if let Some(url) = &res.url {
                println!("      {}", url.bright_blue().underline());
            }
            println!();
        }
    }

    true
}

/// Print a list of all available attack categories for `ai-sec explain`.
pub fn list_explainable_topics() {
    println!();
    println!("{}", "  Available topics for 'ai-sec explain <topic>':".bold());
    println!();
    for attack in all_attacks() {
        println!("    {:25} — {}", attack.id().cyan(), attack.description());
    }
    println!();
}
