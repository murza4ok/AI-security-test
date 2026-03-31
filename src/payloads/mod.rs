//! Payload loading and template rendering.
//!
//! Payloads are stored as TOML files under the `payloads/` directory.
//! This module handles deserialization and variable substitution.

pub mod loader;
pub mod template;

pub use loader::{Payload, PayloadFile, PayloadLoader, PayloadMetadata};
