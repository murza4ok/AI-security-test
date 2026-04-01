//! Terminal display helpers.
//!
//! Centralised colour palette and formatting utilities so that
//! the rest of the codebase doesn't scatter colour codes everywhere.

#![allow(dead_code)]

use owo_colors::OwoColorize;

// ── Colour-coded labels ───────────────────────────────────────────────────────

/// Print a success / "safety held" line
pub fn print_refused(msg: &str) {
    println!("  {} {}", "✓ REFUSED".green().bold(), msg);
}

/// Print an ambiguous / partial result line
pub fn print_partial(msg: &str) {
    println!("  {} {}", "⚠ PARTIAL".yellow().bold(), msg);
}

/// Print a bypass / attack succeeded line
pub fn print_success(msg: &str) {
    println!("  {} {}", "✗ BYPASS ".red().bold(), msg);
}

/// Print an error line
pub fn print_error(msg: &str) {
    println!("  {} {}", "  ERROR  ".bright_red().bold(), msg);
}

/// Print a neutral info line
pub fn print_info(msg: &str) {
    println!("  {} {}", "  INFO   ".bright_blue(), msg);
}

// ── Section headers ───────────────────────────────────────────────────────────

/// Print a prominent banner (used at startup)
pub fn print_banner() {
    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║      AI SECURITY TESTING TOOL  v0.1.0                   ║".cyan());
    println!("{}", "║      Educational LLM Vulnerability Research              ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".cyan());
    println!();
}

/// Print the ethical use disclaimer. Shown once at every startup.
pub fn print_disclaimer() {
    println!("{}", "  ⚠  DISCLAIMER".yellow().bold());
    println!("  This tool is for authorized security testing and education only.");
    println!("  Do not use against any system without explicit permission.");
    println!("  The authors assume no liability for misuse.");
    println!();
}

/// Print a named section header
pub fn print_section(title: &str) {
    println!();
    println!("  {}", format!("── {} ──────────────────────────────", title).bright_blue().bold());
}

/// Print a subsection header (lighter weight)
pub fn print_subsection(title: &str) {
    println!("    {}", title.bold());
}

// ── Utility ───────────────────────────────────────────────────────────────────

/// Truncate a string to `max` chars and append "…" if it was truncated.
/// Used for previewing long LLM responses in the terminal.
pub fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}…", truncated)
    }
}
