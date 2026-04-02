//! Terminal result reporter.
//!
//! Formats session results as coloured tables and text for display in the terminal.

#![allow(dead_code)]

use crate::engine::session::{AttackRun, TestSession};
use comfy_table::{Table, Cell, Color, Attribute};
use owo_colors::OwoColorize;

/// Print a summary table for the entire session.
pub fn print_session_summary(session: &TestSession) {
    println!();
    println!(
        "{}",
        "╔══ ATTACK SUMMARY ═══════════════════════════════════════╗".cyan()
    );
    println!(
        "  Session: {}",
        session.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  Provider: {}", session.provider_name.bold());
    println!("  Attacks run: {}", session.attacks_run.len());
    println!();

    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Attack Category").add_attribute(Attribute::Bold),
        Cell::new("Refused").add_attribute(Attribute::Bold),
        Cell::new("Partial").add_attribute(Attribute::Bold),
        Cell::new("Bypass").add_attribute(Attribute::Bold),
        Cell::new("Info(L0)").add_attribute(Attribute::Bold),
        Cell::new("Bypass %*").add_attribute(Attribute::Bold),
    ]);

    for run in &session.attacks_run {
        // Bypass % is calculated only over L2/L3 payloads (excluding L0 informational)
        let scoreable = run.payloads_tested - run.informational_count;
        let bypass_pct = if scoreable > 0 {
            (run.success_count as f32 / scoreable as f32) * 100.0
        } else {
            0.0
        };
        let bypass_cell = if bypass_pct > 0.0 {
            Cell::new(format!("{:.0}%", bypass_pct)).fg(Color::Red)
        } else {
            Cell::new("0%").fg(Color::Green)
        };

        table.add_row(vec![
            Cell::new(&run.attack_name),
            Cell::new(format!("{}/{}", run.refused_count, run.payloads_tested)).fg(Color::Green),
            Cell::new(format!("{}/{}", run.partial_count, run.payloads_tested)).fg(Color::Yellow),
            Cell::new(format!("{}/{}", run.success_count, run.payloads_tested)).fg(Color::Red),
            Cell::new(format!("{}", run.informational_count)).fg(Color::DarkGrey),
            bypass_cell,
        ]);
    }

    // Totals row — bypass % excludes L0 informational payloads
    let s = &session.summary;
    let scoreable_total = s.total_payloads - s.total_informational;
    let total_pct = if scoreable_total > 0 {
        (s.total_success as f32 / scoreable_total as f32) * 100.0
    } else {
        0.0
    };
    table.add_row(vec![
        Cell::new("TOTAL").add_attribute(Attribute::Bold),
        Cell::new(format!("{}/{}", s.total_refused, s.total_payloads)).fg(Color::Green),
        Cell::new(format!("{}/{}", s.total_partial, s.total_payloads)).fg(Color::Yellow),
        Cell::new(format!("{}/{}", s.total_success, s.total_payloads)).fg(Color::Red),
        Cell::new(format!("{}", s.total_informational)).fg(Color::DarkGrey),
        Cell::new(format!("{:.0}%", total_pct)).add_attribute(Attribute::Bold),
    ]);

    println!("{}", table);
    println!(
        "  {}",
        "* Bypass % считается только по L2/L3 payload'ам (L0 public knowledge исключены)"
            .bright_black()
    );
    println!();
}

/// Print detailed results for a single attack run (truncated preview).
pub fn print_attack_details(run: &AttackRun) {
    println!();
    println!("{}", format!("── {} Details ──", run.attack_name).bold().cyan());
    println!();

    for result in &run.results {
        let label = result.evaluation.label();
        let styled_label = match label {
            "REFUSED" => format!("[{}]", label).green().to_string(),
            "BYPASS"  => format!("[{}]", label).red().bold().to_string(),
            "PARTIAL" => format!("[{}]", label).yellow().to_string(),
            _         => format!("[{}]", label).dimmed().to_string(),
        };

        println!(
            "  {} {} — {}ms",
            styled_label,
            result.payload_name.bold(),
            result.latency_ms
        );

        let preview = crate::cli::display::truncate(&result.response_received, 120);
        println!("     Response: {}", preview.dimmed());
        println!();
    }
}

/// Print a full human-readable review of a session — for manual analysis.
/// Shows the complete response text for every result, nicely formatted.
pub fn print_session_review(session: &TestSession) {
    println!();
    println!("{}", "╔══ REVIEW MODE ══════════════════════════════════════════╗".bright_blue());
    println!(
        "  Session: {}   Provider: {}",
        session.started_at.format("%Y-%m-%d %H:%M UTC"),
        session.provider_name.bold()
    );
    println!("{}", "╚═════════════════════════════════════════════════════════╝".bright_blue());

    for run in &session.attacks_run {
        println!();
        println!(
            "{}",
            format!(
                "  ▶  {}  ({}/{} refused, {}/{} bypass)",
                run.attack_name.bold(),
                run.refused_count, run.payloads_tested,
                run.success_count, run.payloads_tested,
            )
            .cyan()
        );
        println!("  {}", "─".repeat(60).bright_black());

        for (i, result) in run.results.iter().enumerate() {
            let label = result.evaluation.label();
            let styled_label = match label {
                "REFUSED" => format!(" REFUSED ").on_green().black().bold().to_string(),
                "BYPASS"  => format!(" BYPASS  ").on_red().white().bold().to_string(),
                "PARTIAL" => format!(" PARTIAL ").on_yellow().black().bold().to_string(),
                _         => format!(" UNKNOWN ").on_bright_black().white().to_string(),
            };

            println!();
            println!(
                "  {}  {}  {}  {} ms",
                format!("[{}]", i + 1).bright_black(),
                styled_label,
                result.payload_name.bold(),
                result.latency_ms,
            );

            // Show matched evaluation signals
            match &result.evaluation {
                crate::engine::evaluator::EvaluationResult::Refused { matched_phrases, confidence } => {
                    println!(
                        "       {} confidence: {:.0}%  signals: {}",
                        "↳".green(),
                        confidence * 100.0,
                        matched_phrases.join(", ").dimmed()
                    );
                }
                crate::engine::evaluator::EvaluationResult::Success { matched_phrases, confidence } => {
                    println!(
                        "       {} confidence: {:.0}%  signals: {}",
                        "↳".red(),
                        confidence * 100.0,
                        matched_phrases.join(", ").dimmed()
                    );
                }
                crate::engine::evaluator::EvaluationResult::Partial { notes } => {
                    println!("       {} {}", "↳".yellow(), notes.dimmed());
                }
                crate::engine::evaluator::EvaluationResult::Informational => {
                    println!("       {} L0 — public knowledge, answering is correct behaviour", "↳".bright_black());
                }
                crate::engine::evaluator::EvaluationResult::Inconclusive => {
                    println!("       {} no signals matched", "↳".bright_black());
                }
            }

            // Full response text, wrapped at 80 chars
            println!();
            println!("  {}", "PROMPT:".bright_black().bold());
            for line in wrap_text(&result.prompt_sent, 74) {
                println!("    {}", line.bright_black());
            }
            println!();
            println!("  {}", "RESPONSE:".bold());
            if result.response_received.is_empty() {
                println!("    {}", "(empty)".bright_black().italic());
            } else {
                for line in wrap_text(&result.response_received, 74) {
                    println!("    {}", line);
                }
            }
            println!();
            println!("  {}", "─".repeat(60).bright_black());
        }
    }

    println!();
    println!("{}", "  ✓  End of review.".bright_blue().bold());
    println!();
}

/// Wrap text at `width` characters, preserving existing newlines.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        // Simple word-wrap
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current.clone());
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}
