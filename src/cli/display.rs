//! Terminal display helpers.
//!
//! Комментарии и обучающие подсказки держим на русском, чтобы интерфейс для
//! исследования и демонстраций читался единообразно.

use owo_colors::OwoColorize;

pub fn print_refused(msg: &str) {
    println!("  {} {}", "REFUSED".green().bold(), msg);
}

pub fn print_partial(msg: &str) {
    println!("  {} {}", "PARTIAL".yellow().bold(), msg);
}

pub fn print_success(msg: &str) {
    println!("  {} {}", "BYPASS".red().bold(), msg);
}

pub fn print_error(msg: &str) {
    println!("  {} {}", "ERROR".bright_red().bold(), msg);
}

pub fn print_informational(msg: &str) {
    println!("  {} {}", "INFO(L0)".bright_black().bold(), msg);
}

pub fn print_banner() {
    println!();
    println!("{}", "==============================================".cyan());
    println!("{}", "AI SECURITY TESTING TOOL".cyan().bold());
    println!("{}", "Educational LLM Vulnerability Research".cyan());
    println!("{}", "==============================================".cyan());
    println!();
}

pub fn print_disclaimer() {
    println!("{}", "  Внимание".yellow().bold());
    println!("  Инструмент предназначен только для авторизованного тестирования и обучения.");
    println!("  Не используйте его против чужих систем без явного разрешения.");
    println!();
}

pub fn print_section(title: &str) {
    println!();
    println!("  {}", format!("-- {} --", title).bright_blue().bold());
}

pub fn print_usage_hint() {
    println!("  {}", "Краткая справка".bold().bright_blue());
    println!();
    println!("  {}  {}", "ai-sec run -a <category>".cyan(), "запустить одну или несколько атак");
    println!("  {}     {}", "ai-sec list".cyan(), "показать доступные категории атак");
    println!("  {}  {}", "ai-sec explain <id>".cyan(), "показать обучающее описание атаки");
    println!("  {}    {}", "ai-sec check".cyan(), "проверить доступность провайдеров");
    println!("  {} {}", "ai-sec sessions".cyan(), "показать обзор сохранённых сессий");
    println!();
    println!("  Категории:");
    println!("    prompt_injection  jailbreaking  extraction  goal_hijacking");
    println!("    token_attacks     many_shot     context_manipulation");
    println!("    sensitive_data_exposure");
    println!();
    println!("  Провайдер задаётся через `.env` или флаг `--provider`.");
    println!();
}

pub fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{truncated}...")
    }
}
