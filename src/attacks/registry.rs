//! Attack registry.
//!
//! Maps attack category IDs (strings) to concrete implementations.
//! This is the single place where new attack categories must be registered.

use super::{
    Attack,
    context_manipulation::ContextManipulationAttack,
    extraction::ExtractionAttack,
    goal_hijacking::GoalHijackingAttack,
    jailbreaking::JailbreakingAttack,
    many_shot::ManyShotAttack,
    prompt_injection::PromptInjectionAttack,
    token_attacks::TokenAttacksAttack,
};
use std::sync::Arc;

/// Returns all registered attack categories, in the order they appear in menus.
pub fn all_attacks() -> Vec<Arc<dyn Attack>> {
    vec![
        Arc::new(PromptInjectionAttack),
        Arc::new(JailbreakingAttack),
        Arc::new(ExtractionAttack),
        Arc::new(GoalHijackingAttack),
        Arc::new(TokenAttacksAttack),
        Arc::new(ManyShotAttack),
        Arc::new(ContextManipulationAttack),
    ]
}

/// Look up a single attack by its ID string.
/// Returns None if the ID is not recognised.
pub fn find_attack(id: &str) -> Option<Arc<dyn Attack>> {
    all_attacks().into_iter().find(|a| a.id() == id)
}
