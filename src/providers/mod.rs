//! LLM provider implementations.
//!
//! This module defines the core `LLMProvider` trait and implementations for:
//! - OpenAI (GPT-4o, GPT-4-turbo, etc.)
//! - Anthropic (Claude 3.x)
//! - Ollama (local models)
//!
//! The attack engine interacts only with the trait, keeping it provider-agnostic.

pub mod anthropic;
pub mod ollama;
pub mod openai;
pub mod traits;

pub use traits::{LLMProvider, LLMResponse, ProviderError, RequestConfig};
