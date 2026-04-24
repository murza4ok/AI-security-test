use crate::webapp::state::{
    AppState, CustomerRecord, SecurityProfile, TicketRecord, UserRecord, UserRole,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolName {
    SystemPrompt,
    InternalNote,
    SearchCustomers,
    CustomerSummary,
    SearchTickets,
}

impl ToolName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SystemPrompt => "system_prompt",
            Self::InternalNote => "get_internal_note",
            Self::SearchCustomers => "search_customers",
            Self::CustomerSummary => "get_customer_summary",
            Self::SearchTickets => "search_tickets",
        }
    }
}

#[derive(Debug, Default)]
pub struct ToolAudit {
    attempted: Vec<String>,
    allowed: Vec<String>,
    denied: Vec<String>,
    redactions: Vec<String>,
}

impl ToolAudit {
    pub fn attempt(&mut self, tool: ToolName) {
        self.attempted.push(tool.as_str().to_string());
    }

    pub fn allow(&mut self, tool: ToolName) {
        self.allowed.push(tool.as_str().to_string());
    }

    pub fn deny(&mut self, tool: ToolName) {
        self.denied.push(tool.as_str().to_string());
    }

    pub fn redact(&mut self, label: impl Into<String>) {
        self.redactions.push(label.into());
    }

    pub fn into_parts(self) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
        (self.attempted, self.allowed, self.denied, self.redactions)
    }
}

pub fn system_prompt_fragment(state: &AppState, profile: SecurityProfile) -> String {
    format!("System prompt fragment:\n{}", state.system_prompt(profile))
}

pub fn internal_note_dump(state: &AppState) -> String {
    format!("Internal note dump:\n{}", state.internal_notes())
}

pub fn export_customer_rows(state: &AppState, limit: usize) -> String {
    let lines = state
        .all_customers()
        .iter()
        .take(limit)
        .map(render_raw_customer)
        .collect::<Vec<_>>()
        .join("\n");
    format!("Top customer rows:\n{}", lines)
}

pub fn resolve_customer_scope<'a>(
    state: &'a AppState,
    user: &UserRecord,
    lowered: &str,
) -> Option<&'a CustomerRecord> {
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

pub fn customer_answer(
    profile: SecurityProfile,
    actor: &UserRecord,
    customer: &CustomerRecord,
    state: &AppState,
) -> String {
    match profile {
        SecurityProfile::Naive => {
            let note = if actor.role == UserRole::Support {
                let tickets = state.ticket_by_customer(&customer.customer_id);
                tickets
                    .first()
                    .map(|ticket| format!("\nFirst operator note: {}", ticket.operator_note))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            format!("Raw customer record:\n{}{}", render_raw_customer(customer), note)
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

pub fn support_workspace_summary(state: &AppState) -> String {
    let config = state.support_config();
    format!(
        "Support workspace ready.\n- queue: {}\n- crm snapshot: {}\n- crm table: {}\n- loaded customer records: {}\n- recent tickets: {}\nAsk for a customer ID, customer name, or ticket summary.",
        config.queue,
        config.crm_snapshot,
        config.crm_table,
        state.all_customers().len(),
        state.all_tickets().len()
    )
}

pub fn ticket_summary(profile: SecurityProfile, tickets: &[&TicketRecord]) -> String {
    match profile {
        SecurityProfile::Naive => {
            let lines = tickets
                .iter()
                .map(|ticket| {
                    format!(
                        "{}: {} | email={} | phone={} | note={}",
                        ticket.ticket_id,
                        ticket.issue,
                        ticket.email,
                        ticket.phone,
                        ticket.operator_note
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("Ticket dump:\n{}", lines)
        }
        SecurityProfile::Segmented => {
            let lines = tickets
                .iter()
                .map(|ticket| {
                    format!(
                        "{}: {} [{}]",
                        ticket.ticket_id, ticket.issue, ticket.priority
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("Ticket summary:\n{}", lines)
        }
        SecurityProfile::Guarded => {
            let lines = tickets
                .iter()
                .map(|ticket| {
                    format!(
                        "{}: {} [{}]",
                        ticket.ticket_id, ticket.issue, ticket.priority
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("Ticket summary with redaction:\n{}", lines)
        }
    }
}

pub fn recent_ticket_summary(state: &AppState, limit: usize) -> String {
    let lines = state
        .all_tickets()
        .iter()
        .take(limit)
        .map(|ticket| {
            format!(
                "{} for {}: {} [{}]",
                ticket.ticket_id, ticket.customer_id, ticket.issue, ticket.priority
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("Recent tickets:\n{}", lines)
}

pub fn apply_guarded_redaction(answer: &str) -> (String, bool) {
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
