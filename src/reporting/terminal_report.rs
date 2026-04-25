//! Terminal result reporter.

use crate::engine::session::TestSession;
use crate::reporting::json_report::SavedSessionInfo;
use comfy_table::{Attribute, Cell, Color, Table};
use owo_colors::OwoColorize;

pub fn print_session_summary(session: &TestSession) {
    println!();
    println!("{}", "== ATTACK SUMMARY ==".cyan().bold());
    println!(
        "  Session: {}",
        session.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  Provider: {}", session.provider.provider_name.bold());
    println!(
        "  Requested model: {}",
        session.provider.requested_model.bold()
    );
    println!("  Session mode: {}", session_mode_label(session).bold());
    if let Some(target_mode) = &session.target.mode {
        println!("  Target mode: {}", target_mode.bold());
        if let Some(base_url) = &session.target.base_url {
            println!("  Target URL: {}", base_url.bold());
        }
        if let Some(endpoint) = &session.target.endpoint {
            println!("  Target endpoint: {}", endpoint.bold());
        }
        if let Some(user) = &session.target.authenticated_user {
            println!("  Target user: {}", user.bold());
        }
        if let Some(profile) = &session.target.security_profile {
            println!("  Target profile: {}", profile.bold());
        }
        println!("  Target requests: {}", session.target.requests_sent);
        if !session.target.tool_calls_attempted.is_empty() {
            println!(
                "  Target tool calls attempted: {}",
                session.target.tool_calls_attempted.join(", ")
            );
        }
        if !session.target.tool_calls_allowed.is_empty() {
            println!(
                "  Target tool calls allowed: {}",
                session.target.tool_calls_allowed.join(", ")
            );
        }
        if !session.target.tool_calls_denied.is_empty() {
            println!(
                "  Target tool calls denied: {}",
                session.target.tool_calls_denied.join(", ")
            );
        }
        if !session.target.redactions.is_empty() {
            println!(
                "  Target redactions: {}",
                session.target.redactions.join(", ")
            );
        }
    }
    println!("  Attacks run: {}", session.attacks_run.len());
    if session.summary.total_generated_payloads > 0 {
        println!(
            "  Generated payloads: {}",
            session.summary.total_generated_payloads
        );
    }
    if let Some(scenario_name) = &session.scenario.scenario_name {
        println!("  Scenario: {}", scenario_name.bold());
        if let Some(version) = &session.scenario.scenario_version {
            println!("  Scenario version: {}", version.bold());
        }
        if let Some(defense_profile) = &session.scenario.defense_profile {
            println!("  Defense profile: {}", defense_profile.bold());
        }
        println!(
            "  Exposure score: {} | leaked canaries: {} | leaked pii fields: {} | leaked documents: {}",
            session.scenario.exposure_score,
            session.scenario.leaked_canaries.len(),
            session.scenario.leaked_pii_fields.len(),
            session.scenario.leaked_documents.len()
        );
        println!(
            "  Scenario envelopes: real {} | meta {} | secret types {}",
            session.scenario.real_envelopes.len(),
            session.scenario.meta_envelopes.len(),
            session.scenario.leaked_secret_types.len(),
        );
    }
    let (chain_results, completed_chains, stopped_chains, transcript_turns) = chain_stats(session);
    if chain_results > 0 {
        println!(
            "  Multi-turn results: {} | completed {} | stopped {} | transcript turns {}",
            chain_results, completed_chains, stopped_chains, transcript_turns
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
        let bypass_pct = run.bypass_rate_pct;
        table.add_row(vec![
            Cell::new(&run.attack_name),
            Cell::new(format!("{}/{}", run.refused_count, run.payloads_tested)).fg(Color::Green),
            Cell::new(format!("{}/{}", run.partial_count, run.payloads_tested)).fg(Color::Yellow),
            Cell::new(format!("{}/{}", run.success_count, run.payloads_tested)).fg(Color::Red),
            Cell::new(run.informational_count).fg(Color::DarkGrey),
            Cell::new(run.generated_payloads).fg(Color::Cyan),
            Cell::new(format!("{:.0}%", bypass_pct)).fg(if bypass_pct > 0.0 {
                Color::Red
            } else {
                Color::Green
            }),
        ]);
    }

    let summary = &session.summary;
    table.add_row(vec![
        Cell::new("TOTAL").add_attribute(Attribute::Bold),
        Cell::new(format!(
            "{}/{}",
            summary.total_refused, summary.total_payloads
        ))
        .fg(Color::Green),
        Cell::new(format!(
            "{}/{}",
            summary.total_partial, summary.total_payloads
        ))
        .fg(Color::Yellow),
        Cell::new(format!(
            "{}/{}",
            summary.total_success, summary.total_payloads
        ))
        .fg(Color::Red),
        Cell::new(summary.total_informational).fg(Color::DarkGrey),
        Cell::new(summary.total_generated_payloads).fg(Color::Cyan),
        Cell::new(format!("{:.0}%", summary.bypass_rate_pct)).add_attribute(Attribute::Bold),
    ]);

    println!("{}", table);
    println!(
        "  {}",
        "* Bypass % counts only L2/L3 payloads; L0 and L1 are excluded from scoring".bright_black()
    );
    println!();
}

pub fn print_saved_sessions_overview(sessions: &[SavedSessionInfo]) {
    if sessions.is_empty() {
        println!("  No saved sessions found in results/.");
        return;
    }

    println!();
    println!("{}", "== SAVED SESSIONS ==".cyan().bold());
    println!();

    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("#").add_attribute(Attribute::Bold),
        Cell::new("Started").add_attribute(Attribute::Bold),
        Cell::new("Provider").add_attribute(Attribute::Bold),
        Cell::new("Model").add_attribute(Attribute::Bold),
        Cell::new("Mode").add_attribute(Attribute::Bold),
        Cell::new("Attacks").add_attribute(Attribute::Bold),
        Cell::new("Payloads").add_attribute(Attribute::Bold),
        Cell::new("Bypass %").add_attribute(Attribute::Bold),
        Cell::new("Generated").add_attribute(Attribute::Bold),
        Cell::new("Exposure").add_attribute(Attribute::Bold),
        Cell::new("Chains").add_attribute(Attribute::Bold),
        Cell::new("File").add_attribute(Attribute::Bold),
    ]);

    for (index, saved) in sessions.iter().enumerate() {
        let summary = &saved.session.summary;
        let (chain_results, _, _, _) = chain_stats(&saved.session);
        let file_name = saved
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>");

        table.add_row(vec![
            Cell::new(index + 1),
            Cell::new(
                saved
                    .session
                    .started_at
                    .format("%Y-%m-%d %H:%M UTC")
                    .to_string(),
            ),
            Cell::new(&saved.session.provider.provider_name),
            Cell::new(&saved.session.provider.requested_model),
            Cell::new(session_mode_label(&saved.session)),
            Cell::new(saved.session.attacks_run.len()),
            Cell::new(summary.total_payloads),
            Cell::new(format!("{:.0}%", summary.bypass_rate_pct)).fg(
                if summary.bypass_rate_pct > 0.0 {
                    Color::Red
                } else {
                    Color::Green
                },
            ),
            Cell::new(summary.total_generated_payloads).fg(Color::Cyan),
            Cell::new(saved.session.scenario.exposure_score),
            Cell::new(chain_results),
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
    println!("{}", "== SESSION COMPARISON ==".cyan().bold());
    println!();

    let mut overview = Table::new();
    overview.set_header(vec![
        Cell::new("#").add_attribute(Attribute::Bold),
        Cell::new("Provider").add_attribute(Attribute::Bold),
        Cell::new("Model").add_attribute(Attribute::Bold),
        Cell::new("Mode").add_attribute(Attribute::Bold),
        Cell::new("Target/Scenario").add_attribute(Attribute::Bold),
        Cell::new("Payloads").add_attribute(Attribute::Bold),
        Cell::new("Bypass %").add_attribute(Attribute::Bold),
        Cell::new("Exposure").add_attribute(Attribute::Bold),
        Cell::new("Chains").add_attribute(Attribute::Bold),
        Cell::new("Requests").add_attribute(Attribute::Bold),
    ]);
    for (index, session) in sessions.iter().enumerate() {
        let (chain_results, _, _, _) = chain_stats(session);
        overview.add_row(vec![
            Cell::new(index + 1),
            Cell::new(&session.provider.provider_name),
            Cell::new(&session.provider.requested_model),
            Cell::new(session_mode_label(session)),
            Cell::new(target_or_scenario_label(session)),
            Cell::new(session.summary.total_payloads),
            Cell::new(format!("{:.0}%", session.summary.bypass_rate_pct)).fg(
                if session.summary.bypass_rate_pct > 0.0 {
                    Color::Red
                } else {
                    Color::Green
                },
            ),
            Cell::new(session.scenario.exposure_score),
            Cell::new(chain_results),
            Cell::new(session.target.requests_sent),
        ]);
    }
    println!("{}", overview);
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
    for (index, session) in sessions.iter().enumerate() {
        header.push(
            Cell::new(format!("#{} {}", index + 1, session_mode_label(session)))
                .add_attribute(Attribute::Bold),
        );
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

pub fn print_session_review(session: &TestSession) {
    println!();
    println!("{}", "== REVIEW MODE ==".bright_blue().bold());
    println!(
        "  Session: {}   Provider: {}",
        session.started_at.format("%Y-%m-%d %H:%M UTC"),
        session.provider.provider_name.bold()
    );
    println!("  Mode: {}", session_mode_label(session).bold());
    if let Some(target_mode) = &session.target.mode {
        println!(
            "  Target: {} | requests {} | tool attempts {} | denied {} | redactions {}",
            target_mode.bold(),
            session.target.requests_sent,
            session.target.tool_calls_attempted.len(),
            session.target.tool_calls_denied.len(),
            session.target.redactions.len()
        );
    }
    if let Some(scenario_name) = &session.scenario.scenario_name {
        println!(
            "  Scenario: {} | exposure {} | real envelopes {} | meta envelopes {}",
            scenario_name.bold(),
            session.scenario.exposure_score,
            session.scenario.real_envelopes.len(),
            session.scenario.meta_envelopes.len()
        );
    }
    println!("  {}", "-".repeat(64).bright_blue());

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
                    result
                        .seed_payload_id
                        .as_deref()
                        .unwrap_or("<unknown>")
                        .dimmed()
                );
            }
            if !result.evidence.canaries.is_empty() {
                println!(
                    "       -> matched canaries: {}",
                    result.evidence.canaries.join(", ").red()
                );
            }
            if !result.evidence.sensitive_fields.is_empty() {
                println!(
                    "       -> matched sensitive fields: {}",
                    result.evidence.sensitive_fields.join(", ").yellow()
                );
            }
            if !result.evidence.secret_patterns.is_empty() {
                println!(
                    "       -> matched secret patterns: {}",
                    result.evidence.secret_patterns.join(", ").red()
                );
            }
            if !result.evidence.documents.is_empty() {
                println!(
                    "       -> matched documents: {}",
                    result.evidence.documents.join(" | ").yellow()
                );
            }
            if !result.evidence.system_prompt_fragments.is_empty() {
                println!(
                    "       -> matched system prompt fragments: {}",
                    result.evidence.system_prompt_fragments.join(" | ").yellow()
                );
            }
            if !result.evidence.evidence_slices.is_empty() {
                println!(
                    "       -> evidence slices: {}",
                    result
                        .evidence
                        .evidence_slices
                        .iter()
                        .take(3)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" | ")
                        .dimmed()
                );
            }
            if result.damage.score > 0 {
                println!(
                    "       -> damage score: {}",
                    result.damage.score.to_string().red()
                );
                println!(
                    "       -> damage level: {} ({})",
                    format!("{:?}", result.damage.level).red(),
                    result.damage.level.criticality().dimmed()
                );
            }
            if result.damage.score == 0 && !result.evidence.is_empty() {
                println!(
                    "       -> evidence captured without score escalation: {}",
                    result.damage.summary.dimmed()
                );
            }
            println!("       -> confidence: {:.0}%", result.confidence * 100.0);
            println!(
                "       -> requires review: {}",
                if result.requires_review {
                    "yes".yellow().to_string()
                } else {
                    "no".green().to_string()
                }
            );
            if result.chain_planned_turns > 1 {
                let status = if result.chain_completed {
                    "completed".green().to_string()
                } else {
                    "stopped early".yellow().to_string()
                };
                println!(
                    "       -> chain: {}/{} turns ({})",
                    result.chain_executed_turns, result.chain_planned_turns, status
                );
                if let Some(reason) = &result.chain_abort_reason {
                    println!("       -> chain stop reason: {}", reason.dimmed());
                }
            }
            if !result.rationale.is_empty() {
                println!("       -> rationale: {}", result.rationale.dimmed());
            }

            match &result.evaluation {
                crate::engine::evaluator::EvaluationResult::Refused {
                    matched_phrases, ..
                } => {
                    println!(
                        "       -> refusal signals: {}",
                        matched_phrases.join(", ").dimmed()
                    );
                }
                crate::engine::evaluator::EvaluationResult::Success {
                    matched_phrases, ..
                } => {
                    println!(
                        "       -> bypass signals: {}",
                        matched_phrases.join(", ").dimmed()
                    );
                }
                crate::engine::evaluator::EvaluationResult::Partial { notes } => {
                    println!("       -> partial notes: {}", notes.dimmed());
                }
                crate::engine::evaluator::EvaluationResult::Informational => {
                    println!("{}", "       -> L0 informational response".dimmed());
                }
                crate::engine::evaluator::EvaluationResult::Inconclusive => {
                    println!("{}", "       -> no stable leak signal matched".dimmed());
                }
            }

            println!();
            if result.transcript.len() > 1 {
                println!("  {}", "TRANSCRIPT:".bright_black().bold());
                for turn in &result.transcript {
                    println!(
                        "    {} {}",
                        format!("Turn {}", turn.step_index).bright_black().bold(),
                        turn.user_message.dimmed()
                    );
                    println!("      sent:");
                    for line in wrap_text(&turn.prompt_sent, 70) {
                        println!("        {}", line.bright_black());
                    }
                    println!("      response:");
                    if turn.response_received.is_empty() {
                        println!("        {}", "(empty)".bright_black().italic());
                    } else {
                        for line in wrap_text(&turn.response_received, 70) {
                            println!("        {}", line);
                        }
                    }
                }
            } else {
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
            }
            println!();
            println!("  {}", "-".repeat(60).bright_black());
        }
    }

    println!();
    println!("{}", "  End of review.".bright_blue().bold());
    println!();
}

fn session_mode_label(session: &TestSession) -> String {
    let mut parts = Vec::new();
    if session.scenario.scenario_name.is_some() {
        parts.push("scenario");
    }
    if session.target.mode.is_some() {
        parts.push("http-target");
    }
    let (chain_results, _, _, _) = chain_stats(session);
    if chain_results > 0 {
        parts.push("multi-turn");
    }
    if parts.is_empty() {
        parts.push("direct");
    }
    parts.join("+")
}

fn target_or_scenario_label(session: &TestSession) -> String {
    if let Some(target_url) = &session.target.base_url {
        if let Some(profile) = &session.target.security_profile {
            return format!("{} ({})", target_url, profile);
        }
        return target_url.clone();
    }
    if let Some(scenario_name) = &session.scenario.scenario_name {
        return scenario_name.clone();
    }
    "-".to_string()
}

fn chain_stats(session: &TestSession) -> (usize, usize, usize, usize) {
    let mut chain_results = 0;
    let mut completed = 0;
    let mut stopped = 0;
    let mut transcript_turns = 0;

    for run in &session.attacks_run {
        for result in &run.results {
            transcript_turns += result.transcript.len();
            if result.chain_planned_turns > 1 {
                chain_results += 1;
                if result.chain_completed {
                    completed += 1;
                } else {
                    stopped += 1;
                }
            }
        }
    }

    (chain_results, completed, stopped, transcript_turns)
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
