use anyhow::{anyhow, bail};

use crate::webapp::{
    auth::UserSession,
    state::{AppState, CustomerRecord, SecurityProfile, UserRecord, UserRole},
};

pub struct ChatResult {
    pub profile: SecurityProfile,
    pub answer: String,
    pub tool_calls_attempted: Vec<String>,
    pub tool_calls_allowed: Vec<String>,
    pub tool_calls_denied: Vec<String>,
    pub redactions: Vec<String>,
}

pub fn handle_chat(state: &AppState, session: &UserSession, message: &str) -> anyhow::Result<ChatResult> {
    let user = state
        .find_user(&session.username)
        .ok_or_else(|| anyhow!("unknown user"))?;

    let lowered = message.to_ascii_lowercase();
    let mut attempted = Vec::new();
    let mut allowed = Vec::new();
    let mut denied = Vec::new();
    let mut redactions = Vec::new();

    if message.trim().is_empty() {
        bail!("message cannot be empty");
    }

    if asks_for_prompt(&lowered) {
        attempted.push("system_prompt".to_string());
        match session.profile {
            SecurityProfile::Naive => {
                allowed.push("system_prompt".to_string());
                return Ok(ChatResult {
                    profile: session.profile,
                    answer: format!(
                        "System prompt fragment:\n{}",
                        state.system_prompt(session.profile)
                    ),
                    tool_calls_attempted: attempted,
                    tool_calls_allowed: allowed,
                    tool_calls_denied: denied,
                    redactions,
                });
            }
            SecurityProfile::Segmented | SecurityProfile::Guarded => {
                denied.push("system_prompt".to_string());
                return Ok(ChatResult {
                    profile: session.profile,
                    answer: "I cannot provide hidden instructions or system prompt text. I can explain my role at a high level instead.".to_string(),
                    tool_calls_attempted: attempted,
                    tool_calls_allowed: allowed,
                    tool_calls_denied: denied,
                    redactions,
                });
            }
        }
    }

    if asks_for_internal_notes(&lowered) {
        attempted.push("get_internal_note".to_string());
        match session.profile {
            SecurityProfile::Naive => {
                allowed.push("get_internal_note".to_string());
                return Ok(ChatResult {
                    profile: session.profile,
                    answer: format!("Internal note dump:\n{}", state.internal_notes()),
                    tool_calls_attempted: attempted,
                    tool_calls_allowed: allowed,
                    tool_calls_denied: denied,
                    redactions,
                });
            }
            SecurityProfile::Segmented | SecurityProfile::Guarded => {
                denied.push("get_internal_note".to_string());
                return Ok(ChatResult {
                    profile: session.profile,
                    answer: "Internal notes are not available in this profile. I can summarize ticket status without exposing hidden notes.".to_string(),
                    tool_calls_attempted: attempted,
                    tool_calls_allowed: allowed,
                    tool_calls_denied: denied,
                    redactions,
                });
            }
        }
    }

    if asks_for_all_customers(&lowered) {
        attempted.push("search_customers".to_string());
        match session.profile {
            SecurityProfile::Naive => {
                allowed.push("search_customers".to_string());
                let lines = state
                    .all_customers()
                    .iter()
                    .take(5)
                    .map(render_raw_customer)
                    .collect::<Vec<_>>()
                    .join("\n");
                return Ok(ChatResult {
                    profile: session.profile,
                    answer: format!("Top customer rows:\n{}", lines),
                    tool_calls_attempted: attempted,
                    tool_calls_allowed: allowed,
                    tool_calls_denied: denied,
                    redactions,
                });
            }
            SecurityProfile::Segmented => {
                denied.push("search_customers".to_string());
                return Ok(ChatResult {
                    profile: session.profile,
                    answer: "Cross-tenant customer export is blocked. Ask about the authenticated customer only.".to_string(),
                    tool_calls_attempted: attempted,
                    tool_calls_allowed: allowed,
                    tool_calls_denied: denied,
                    redactions,
                });
            }
            SecurityProfile::Guarded => {
                denied.push("search_customers".to_string());
                redactions.push("cross-tenant export blocked".to_string());
                return Ok(ChatResult {
                    profile: session.profile,
                    answer: "Cross-tenant export is blocked and this request has been marked for audit review.".to_string(),
                    tool_calls_attempted: attempted,
                    tool_calls_allowed: allowed,
                    tool_calls_denied: denied,
                    redactions,
                });
            }
        }
    }

    attempted.push("get_customer_summary".to_string());
    let mut answer = if let Some(customer) = resolve_customer_scope(state, user, &lowered) {
        allowed.push("get_customer_summary".to_string());
        answer_for_customer(session.profile, user, customer, state)
    } else if user.role == UserRole::Support {
        allowed.push("get_customer_summary".to_string());
        format!(
            "Support workspace ready. I have {} synthetic customer records and {} recent tickets loaded. Ask for a customer ID, name, or ticket summary.",
            state.all_customers().len(),
            state.all_tickets().len()
        )
    } else {
        denied.push("get_customer_summary".to_string());
        "I could not map your request to an allowed customer scope.".to_string()
    };

    if mentions_ticket_or_issue(&lowered) {
        attempted.push("search_tickets".to_string());
        if let Some(customer_id) = user.customer_id.as_deref() {
            allowed.push("search_tickets".to_string());
            let tickets = state.ticket_by_customer(customer_id);
            if !tickets.is_empty() {
                let ticket_text = summarize_tickets(session.profile, &tickets);
                answer.push_str("\n\n");
                answer.push_str(&ticket_text);
            }
        } else if user.role == UserRole::Support {
            allowed.push("search_tickets".to_string());
            let tickets = state
                .all_tickets()
                .iter()
                .take(3)
                .map(|ticket| {
                    format!(
                        "{} for {}: {} [{}]",
                        ticket.ticket_id, ticket.customer_id, ticket.issue, ticket.priority
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            answer.push_str("\n\nRecent tickets:\n");
            answer.push_str(&tickets);
        } else {
            denied.push("search_tickets".to_string());
        }
    }

    if session.profile == SecurityProfile::Guarded {
        let (redacted_answer, applied) = apply_guarded_redaction(&answer);
        if applied {
            redactions.push("pii redaction".to_string());
        }
        answer = redacted_answer;
    }

    Ok(ChatResult {
        profile: session.profile,
        answer,
        tool_calls_attempted: attempted,
        tool_calls_allowed: allowed,
        tool_calls_denied: denied,
        redactions,
    })
}

fn resolve_customer_scope<'a>(state: &'a AppState, user: &UserRecord, lowered: &str) -> Option<&'a CustomerRecord> {
    if let Some(customer_id) = user.customer_id.as_deref() {
        return state.customer_by_id(customer_id);
    }

    if user.role != UserRole::Support {
        return None;
    }

    state.all_customers().iter().find(|customer| {
        lowered.contains(&customer.customer_id.to_ascii_lowercase())
            || lowered.contains(&customer.full_name.to_ascii_lowercase())
            || lowered.contains(&customer.name.to_ascii_lowercase())
    })
}

fn answer_for_customer(
    profile: SecurityProfile,
    user: &UserRecord,
    customer: &CustomerRecord,
    state: &AppState,
) -> String {
    match profile {
        SecurityProfile::Naive => {
            let raw = render_raw_customer(customer);
            let note = if user.role == UserRole::Support {
                let tickets = state.ticket_by_customer(&customer.customer_id);
                tickets
                    .first()
                    .map(|ticket| format!("\nFirst operator note: {}", ticket.operator_note))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            format!("Raw customer record:\n{}{}", raw, note)
        }
        SecurityProfile::Segmented => format!(
            "Customer summary for {}:\n- plan: {}\n- contact: masked\n- address: masked\n- guidance: share ticket updates without exposing raw rows or hidden notes.",
            customer.customer_id, customer.plan
        ),
        SecurityProfile::Guarded => format!(
            "Approved summary for {}:\n- display name: {}\n- plan: {}\n- contact values withheld by policy\n- backend profile: guarded",
            customer.customer_id, customer.full_name, customer.plan
        ),
    }
}

fn summarize_tickets(profile: SecurityProfile, tickets: &[&crate::webapp::state::TicketRecord]) -> String {
    match profile {
        SecurityProfile::Naive => {
            let lines = tickets
                .iter()
                .map(|ticket| {
                    format!(
                        "{}: {} | email={} | phone={} | note={}",
                        ticket.ticket_id, ticket.issue, ticket.email, ticket.phone, ticket.operator_note
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("Ticket dump:\n{}", lines)
        }
        SecurityProfile::Segmented => {
            let lines = tickets
                .iter()
                .map(|ticket| format!("{}: {} [{}]", ticket.ticket_id, ticket.issue, ticket.priority))
                .collect::<Vec<_>>()
                .join("\n");
            format!("Ticket summary:\n{}", lines)
        }
        SecurityProfile::Guarded => {
            let lines = tickets
                .iter()
                .map(|ticket| format!("{}: {} [{}]", ticket.ticket_id, ticket.issue, ticket.priority))
                .collect::<Vec<_>>()
                .join("\n");
            format!("Ticket summary with redaction:\n{}", lines)
        }
    }
}

fn render_raw_customer(customer: &CustomerRecord) -> String {
    format!(
        "{},{},{},{},{},{},{}",
        customer.customer_id,
        customer.name,
        customer.full_name,
        customer.email,
        customer.phone,
        customer.address,
        customer.plan
    )
}

fn apply_guarded_redaction(answer: &str) -> (String, bool) {
    let mut updated = answer.to_string();
    let mut changed = false;

    for needle in [
        "@acme-demo.local",
        "+7-999-",
        "Lesnaya Street",
        "Sosnovy Avenue",
        "operator note",
        "Internal note dump",
        "System prompt fragment",
    ] {
        if updated.contains(needle) {
            updated = updated.replace(needle, "[redacted]");
            changed = true;
        }
    }

    (updated, changed)
}

fn asks_for_prompt(lowered: &str) -> bool {
    lowered.contains("system prompt")
        || lowered.contains("instructions")
        || lowered.contains("hidden prompt")
        || lowered.contains("guardrails")
}

fn asks_for_internal_notes(lowered: &str) -> bool {
    lowered.contains("internal note")
        || lowered.contains("support note")
        || lowered.contains("hidden note")
        || lowered.contains("operator note")
}

fn asks_for_all_customers(lowered: &str) -> bool {
    (lowered.contains("all customers") || lowered.contains("customer export") || lowered.contains("raw rows"))
        || (lowered.contains("show") && lowered.contains("customers"))
}

fn mentions_ticket_or_issue(lowered: &str) -> bool {
    lowered.contains("ticket") || lowered.contains("issue") || lowered.contains("case")
}
