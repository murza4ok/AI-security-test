use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{anyhow, Context};
use serde::Deserialize;

use crate::webapp::auth::UserSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityProfile {
    Naive,
    Segmented,
    Guarded,
}

impl SecurityProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Naive => "naive",
            Self::Segmented => "segmented",
            Self::Guarded => "guarded",
        }
    }

    pub fn as_cookie_value(self) -> &'static str {
        self.as_str()
    }

    pub fn from_form_value(value: &str) -> anyhow::Result<Self> {
        match value {
            "naive" => Ok(Self::Naive),
            "segmented" => Ok(Self::Segmented),
            "guarded" => Ok(Self::Guarded),
            other => Err(anyhow!("unsupported security profile: {}", other)),
        }
    }
}

impl std::fmt::Display for SecurityProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Guest,
    Customer,
    Support,
}

impl UserRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Guest => "guest",
            Self::Customer => "customer",
            Self::Support => "support",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub username: String,
    pub role: UserRole,
    pub customer_id: Option<String>,
    pub tenant: Option<String>,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct CustomerRecord {
    pub customer_id: String,
    pub name: String,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub plan: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TicketRecord {
    pub ticket_id: String,
    pub customer_id: String,
    pub email: String,
    pub phone: String,
    pub issue: String,
    pub priority: String,
    pub operator_note: String,
}

#[derive(Debug, Clone)]
pub struct TranscriptTurn {
    pub user_message: String,
    pub assistant_message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SupportConfig {
    pub queue: String,
    pub crm_snapshot: String,
    pub crm_table: String,
}

#[derive(Debug, Clone)]
struct SupportDataset {
    support_config: SupportConfig,
    customers: Vec<CustomerRecord>,
    tickets: Vec<TicketRecord>,
    internal_notes: String,
    system_prompt: String,
    hardened_system_prompt: String,
}

pub struct AppState {
    users: Vec<UserRecord>,
    dataset: SupportDataset,
    transcripts: Mutex<HashMap<String, Vec<TranscriptTurn>>>,
}

impl AppState {
    pub fn load_from_repo() -> anyhow::Result<Self> {
        let fixture_dir = PathBuf::from("fixtures/sensitive_data_exposure/support_bot");
        let hardened_dir = PathBuf::from("fixtures/sensitive_data_exposure/support_bot_hardened");

        let support_config = load_support_config(&fixture_dir.join("support_config.toml"))?;
        let customers = load_customers(&fixture_dir.join("customers.csv"))?;
        let tickets = load_tickets(&fixture_dir.join("tickets.json"))?;
        let internal_notes = fs::read_to_string(fixture_dir.join("support_notes.md"))
            .context("failed to read notes")?;
        let system_prompt = fs::read_to_string(fixture_dir.join("system_prompt.txt"))
            .context("failed to read system prompt")?;
        let hardened_system_prompt = fs::read_to_string(hardened_dir.join("system_prompt.txt"))
            .context("failed to read hardened system prompt")?;

        Ok(Self {
            users: demo_users(),
            dataset: SupportDataset {
                support_config,
                customers,
                tickets,
                internal_notes,
                system_prompt,
                hardened_system_prompt,
            },
            transcripts: Mutex::new(HashMap::new()),
        })
    }

    pub fn users(&self) -> &[UserRecord] {
        &self.users
    }

    pub fn find_user(&self, username: &str) -> Option<&UserRecord> {
        self.users.iter().find(|user| user.username == username)
    }

    pub fn transcript_for(&self, session: &UserSession) -> Vec<TranscriptTurn> {
        self.transcripts
            .lock()
            .expect("transcript lock poisoned")
            .get(&session_key(session))
            .cloned()
            .unwrap_or_default()
    }

    pub fn append_turn(&self, session: &UserSession, user_message: &str, assistant_message: &str) {
        let mut transcripts = self.transcripts.lock().expect("transcript lock poisoned");
        transcripts
            .entry(session_key(session))
            .or_default()
            .push(TranscriptTurn {
                user_message: user_message.to_string(),
                assistant_message: assistant_message.to_string(),
            });
    }

    pub fn all_customers(&self) -> &[CustomerRecord] {
        &self.dataset.customers
    }

    pub fn customer_by_id(&self, customer_id: &str) -> Option<&CustomerRecord> {
        self.dataset
            .customers
            .iter()
            .find(|record| record.customer_id == customer_id)
    }

    pub fn ticket_by_customer(&self, customer_id: &str) -> Vec<&TicketRecord> {
        self.dataset
            .tickets
            .iter()
            .filter(|ticket| ticket.customer_id == customer_id)
            .collect()
    }

    pub fn all_tickets(&self) -> &[TicketRecord] {
        &self.dataset.tickets
    }

    pub fn internal_notes(&self) -> &str {
        &self.dataset.internal_notes
    }

    pub fn system_prompt(&self, profile: SecurityProfile) -> &str {
        match profile {
            SecurityProfile::Naive => &self.dataset.system_prompt,
            SecurityProfile::Segmented | SecurityProfile::Guarded => {
                &self.dataset.hardened_system_prompt
            }
        }
    }

    pub fn support_config(&self) -> &SupportConfig {
        &self.dataset.support_config
    }
}

fn session_key(session: &UserSession) -> String {
    format!("{}:{}", session.username, session.profile.as_str())
}

fn demo_users() -> Vec<UserRecord> {
    vec![
        UserRecord {
            username: "guest".to_string(),
            role: UserRole::Guest,
            customer_id: None,
            tenant: None,
            display_name: "Guest Operator".to_string(),
        },
        UserRecord {
            username: "customer_alice".to_string(),
            role: UserRole::Customer,
            customer_id: Some("CUST-1001".to_string()),
            tenant: Some("tenant-red".to_string()),
            display_name: "Alice Smirnova".to_string(),
        },
        UserRecord {
            username: "customer_bob".to_string(),
            role: UserRole::Customer,
            customer_id: Some("CUST-1002".to_string()),
            tenant: Some("tenant-blue".to_string()),
            display_name: "Bob Volkov".to_string(),
        },
        UserRecord {
            username: "agent_support".to_string(),
            role: UserRole::Support,
            customer_id: None,
            tenant: Some("ops".to_string()),
            display_name: "Support Agent".to_string(),
        },
    ]
}

fn load_support_config(path: &Path) -> anyhow::Result<SupportConfig> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let config =
        toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(config)
}

fn load_customers(path: &Path) -> anyhow::Result<Vec<CustomerRecord>> {
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let mut customers = Vec::new();
    for row in reader.records() {
        let row = row?;
        customers.push(CustomerRecord {
            customer_id: row.get(0).unwrap_or_default().to_string(),
            name: row.get(1).unwrap_or_default().to_string(),
            full_name: row.get(2).unwrap_or_default().to_string(),
            email: row.get(3).unwrap_or_default().to_string(),
            phone: row.get(4).unwrap_or_default().to_string(),
            address: row.get(5).unwrap_or_default().to_string(),
            plan: row.get(6).unwrap_or_default().to_string(),
        });
    }
    Ok(customers)
}

fn load_tickets(path: &Path) -> anyhow::Result<Vec<TicketRecord>> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let tickets = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(tickets)
}
