//! Terminal result reporter.

#![allow(dead_code)]

use crate::engine::session::{AttackRun, TestSession};
use crate::reporting::json_report::SavedSessionInfo;
use comfy_table::{Attribute, Cell, Color, Table};
use owo_colors::OwoColorize;

pub fn print_session_summary(session: &TestSession) {
    println!();
    println!("{}", "╔══ ATTACK SUMMARY ════════════════════════════════".cyan());
    println!(
        "  Session: {}",
        session.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  Provider: {}", session.provider.provider_name.bold());
    println!(
        "  Requested model: {}",
        session.provider.requested_model.bold()
    );
    println!("  Attacks run: {}", session.attacks_run.len());
    if session.summary.total_generated_payloads > 0 {
        println!(
            "  Generated payloads: {}",
            session.summary.total_generated_payloads
        );
    }
    if let Some(scenario_name) = &session.scenario.scenario_name {
        println!("  Scenario: {}", scenario_name.bold());
        println!(
            "  Exposure score: {} | leaked canaries: {} | leaked pii fields: {} | leaked documents: {}",
            session.scenario.exposure_score,
            session.scenario.leaked_canaries.len(),
            session.scenario.leaked_pii_fields.len(),
            session.scenario.leaked_documents.len()
        );
    }
    println!();

    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Attack Category").add_attribute(Attribute::Bold),
        Cell::new("Refused").add_attribute(Attribute::Bold),
        Cell::new("Partial").add_attribute(Attribute::Bold),
        Cell::new("Bypass").add_attribute(Attribute::Bold),
        Cell::new("Info(L0)").add_attribute(Attribute::Bold),
        Cell::new("Generated").add_attribute(Attribute::Bold),
        Cell::new("Bypass %").add_attribute(Attribute::Bold),
    ]);

    for run in &session.attacks_run {
        let bypass_pct = run.bypass_rate_pct();
        table.add_row(vec![
            Cell::new(&run.attack_name),
            Cell::new(format!("{}/{}", run.refused_count, run.payloads_tested)).fg(Color::Green),
            Cell::new(format!("{}/{}", run.partial_count, run.payloads_tested)).fg(Color::Yellow),
            Cell::new(format!("{}/{}", run.success_count, run.payloads_tested)).fg(Color::Red),
            Cell::new(run.informational_count).fg(Color::DarkGrey),
            Cell::new(run.generated_payloads).fg(Color::Cyan),
            Cell::new(format!("{:.0}%", bypass_pct))
                .fg(if bypass_pct > 0.0 { Color::Red } else { Color::Green }),
        ]);
    }

    let summary = &session.summary;
    table.add_row(vec![
        Cell::new("TOTAL").add_attribute(Attribute::Bold),
        Cell::new(format!("{}/{}", summary.total_refused, summary.total_payloads))
            .fg(Color::Green),
        Cell::new(format!("{}/{}", summary.total_partial, summary.total_payloads))
            .fg(Color::Yellow),
        Cell::new(format!("{}/{}", summary.total_success, summary.total_payloads)).fg(Color::Red),
        Cell::new(summary.total_informational).fg(Color::DarkGrey),
        Cell::new(summary.total_generated_payloads).fg(Color::Cyan),
        Cell::new(format!("{:.0}%", summary.bypass_rate_pct)).add_attribute(Attribute::Bold),
    ]);

    println!("{}", table);
    println!(
        "  {}",
        "* Bypass % counts only L2/L3 payloads; L0 and L1 are excluded from scoring"
            .bright_black()
    );
    println!();
}

pub fn print_saved_sessions_overview(sessions: &[SavedSessionInfo]) {
    if sessions.is_empty() {
        println!("  No saved sessions found in results/.");
        return;
    }

    println!();
    println!("{}", "╔══ SAVED SESSIONS ════════════════════════════════".cyan());
    println!();

    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("#").add_attribute(Attribute::Bold),
        Cell::new("Started").add_attribute(Attribute::Bold),
        Cell::new("Provider").add_attribute(Attribute::Bold),
        Cell::new("Model").add_attribute(Attribute::Bold),
        Cell::new("Attacks").add_attribute(Attribute::Bold),
        Cell::new("Payloads").add_attribute(Attribute::Bold),
        Cell::new("Generated").add_attribute(Attribute::Bold),
        Cell::new("Exposure").add_attribute(Attribute::Bold),
        Cell::new("File").add_attribute(Attribute::Bold),
    ]);

    for (index, saved) in sessions.iter().enumerate() {
        let summary = &saved.session.summary;
        let file_name = saved
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>");

        table.add_row(vec![
            Cell::new(index + 1),
            Cell::new(saved.session.started_at.format("%Y-%m-%d %H:%M UTC").to_string()),
            Cell::new(&saved.session.provider.provider_name),
            Cell::new(&saved.session.provider.requested_model),
            Cell::new(saved.session.attacks_run.len()),
            Cell::new(summary.total_payloads),
            Cell::new(summary.total_generated_payloads).fg(Color::Cyan),
            Cell::new(saved.session.scenario.exposure_score),
            Cell::new(file_name),
        ]);
    }

    println!("{}", table);
    println!();
}

pub fn print_comparison_table(sessions: &[TestSession]) {
    if sessions.is_empty() {
        println!("  No sessions available for comparison.");
        return;
    }

    println!();
    println!("{}", "╔══ SESSION COMPARISON ════════════════════════════".cyan());
    println!();

    for (index, session) in sessions.iter().enumerate() {
        let exposure_suffix = if session.scenario.exposure_score > 0 {
            format!(" | exposure {}", session.scenario.exposure_score)
        } else {
            String::new()
        };
        println!(
            "  [{}] {}   {}{}",
            index + 1,
            session.provider.provider_name.bold(),
            session
                .started_at
                .format("%Y-%m-%d %H:%M UTC")
                .to_string()
                .bright_black(),
            exposure_suffix.bright_black()
        );
    }
    println!();

    let mut all_categories: Vec<(String, String)> = Vec::new();
    for session in sessions {
        for run in &session.attacks_run {
            if !all_categories.iter().any(|(id, _)| id == &run.attack_id) {
                all_categories.push((run.attack_id.clone(), run.attack_name.clone()));
            }
        }
    }

    let mut table = Table::new();
    let mut header = vec![Cell::new("Attack Category").add_attribute(Attribute::Bold)];
    for session in sessions {
        header.push(Cell::new(&session.provider.provider_name).add_attribute(Attribute::Bold));
    }
    table.set_header(header);

    for (attack_id, attack_name) in &all_categories {
        let mut row = vec![Cell::new(attack_name)];
        for session in sessions {
            if let Some(run) = session
                .attacks_run
                .iter()
                .find(|run| &run.attack_id == attack_id)
            {
                row.push(
                    Cell::new(format!(
                        "{:.0}% ({}/{})",
                        run.bypass_rate_pct, run.success_count, run.payloads_tested
                    ))
                    .fg(if run.bypass_rate_pct > 0.0 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                );
            } else {
                row.push(Cell::new("-").fg(Color::DarkGrey));
            }
        }
        table.add_row(row);
    }

    println!("{}", table);
    println!();
}

pub fn print_attack_details(run: &AttackRun) {
    println!();
    println!("{}", format!("-- {} Details --", run.attack_name).bold().cyan());
    println!();

    for result in &run.results {
        println!(
            "  [{}] {} - {}ms",
            result.evaluation.label(),
            result.payload_name.bold(),
            result.latency_ms
        );
        let preview = crate::cli::display::truncate(&result.response_received, 120);
        println!("     Response: {}", preview.dimmed());
        println!();
    }
}

pub fn print_session_review(session: &TestSession) {
    println!();
    println!("{}", "╔══ REVIEW MODE ═══════════════════════════════════".bright_blue());
    println!(
        "  Session: {}   Provider: {}",
        session.started_at.format("%Y-%m-%d %H:%M UTC"),
        session.provider.provider_name.bold()
    );
    if let Some(scenario_name) = &session.scenario.scenario_name {
        println!(
            "  Scenario: {} | exposure {}",
            scenario_name.bold(),
            session.scenario.exposure_score
        );
    }
    println!("{}", "╚══════════════════════════════════════════════════".bright_blue());

    for run in &session.attacks_run {
        println!();
        println!(
            "{}",
            format!(
                "  > {}  ({}/{} refused, {}/{} bypass)",
                run.attack_name.bold(),
                run.refused_count,
                run.payloads_tested,
                run.success_count,
                run.payloads_tested,
            )
            .cyan()
        );
        println!("  {}", "-".repeat(60).bright_black());

        for (index, result) in run.results.iter().enumerate() {
            println!(
                "  [{}] {}  {}  {} ms",
                index + 1,
                result.evaluation.label().bold(),
                result.payload_name.bold(),
                result.latency_ms,
            );
            if result.generated {
                println!(
                    "       -> generated from seed {}",
                    result.seed_payload_id.as_deref().unwrap_or("<unknown>").dimmed()
                );
            }
            if !result.matched_canaries.is_empty() {
                println!("       -> matched canaries: {}", result.matched_canaries.join(", ").red());
            }
            if !result.matched_sensitive_fields.is_empty() {
                println!(
                    "       -> matched sensitive fields: {}",
                    result.matched_sensitive_fields.join(", ").yellow()
                );
            }
            if !result.matched_secret_patterns.is_empty() {
                println!(
                    "       -> matched secret patterns: {}",
                    result.matched_secret_patterns.join(", ").red()
                );
            }
            if !result.matched_documents.is_empty() {
                println!(
                    "       -> matched documents: {}",
                    result.matched_documents.join(" | ").yellow()
                );
            }
            if !result.matched_system_prompt_fragments.is_empty() {
                println!(
                    "       -> matched system prompt fragments: {}",
                    result.matched_system_prompt_fragments.join(" | ").yellow()
                );
            }
            if result.exposure_score > 0 {
                println!("       -> exposure score: {}", result.exposure_score.to_string().red());
            }

            match &result.evaluation {
                crate::engine::evaluator::EvaluationResult::Refused {
                    matched_phrases,
                    confidence,
                } => {
                    println!(
                        "       -> confidence: {:.0}%  signals: {}",
                        confidence * 100.0,
                        matched_phrases.join(", ").dimmed()
                    );
                }
                crate::engine::evaluator::EvaluationResult::Success {
                    matched_phrases,
                    confidence,
                } => {
                    println!(
                        "       -> confidence: {:.0}%  bypass reason: {}",
                        confidence * 100.0,
                        matched_phrases.join(", ").dimmed()
                    );
                }
                crate::engine::evaluator::EvaluationResult::Partial { notes } => {
                    println!("       -> {}", notes.dimmed());
                }
                crate::engine::evaluator::EvaluationResult::Informational => {
                    println!("{}", "       -> L0 informational response".dimmed());
                }
                crate::engine::evaluator::EvaluationResult::Inconclusive => {
                    println!("{}", "       -> no stable leak signal matched".dimmed());
                }
            }

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
            println!("  {}", "-".repeat(60).bright_black());
        }
    }

    println!();
    println!("{}", "  End of review.".bright_blue().bold());
    println!();
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

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
