//! Terminal result reporter.
//!
//! Formats session results as coloured tables and text for display in the terminal.

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
        Cell::new("Bypass %").add_attribute(Attribute::Bold),
    ]);

    for run in &session.attacks_run {
        let bypass_pct = run.bypass_rate_pct();
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
            bypass_cell,
        ]);
    }

    // Totals row
    let s = &session.summary;
    let total_pct = if s.total_payloads > 0 {
        (s.total_success as f32 / s.total_payloads as f32) * 100.0
    } else {
        0.0
    };
    table.add_row(vec![
        Cell::new("TOTAL").add_attribute(Attribute::Bold),
        Cell::new(format!("{}/{}", s.total_refused, s.total_payloads)).fg(Color::Green),
        Cell::new(format!("{}/{}", s.total_partial, s.total_payloads)).fg(Color::Yellow),
        Cell::new(format!("{}/{}", s.total_success, s.total_payloads)).fg(Color::Red),
        Cell::new(format!("{:.0}%", total_pct)).add_attribute(Attribute::Bold),
    ]);

    println!("{}", table);
}

/// Print detailed results for a single attack run.
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

        // Print a truncated preview of the response
        let preview = crate::cli::display::truncate(&result.response_received, 120);
        println!("     Response: {}", preview.dimmed());
        println!();
    }
}
