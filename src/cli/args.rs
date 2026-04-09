//! CLI argument definitions using clap.
//!
//! Комментарии и справка для пользователя держатся на русском языке.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "ai-sec",
    about = "CLI-инструмент для тестирования безопасности LLM и LLM-приложений",
    long_about = "ai-sec помогает исследовать уязвимости LLM, проводить сценарные red-team прогоны, запускать генеративные атаки и сравнивать результаты между моделями и провайдерами.\nИнструмент предназначен только для учебного и авторизованного тестирования.",
    after_help = "Быстрые примеры:\n  ai-sec list\n  ai-sec run --attack jailbreaking --provider deepseek\n  ai-sec run --attack prompt_injection --provider deepseek --generated 3\n  ai-sec run --attack sensitive_data_exposure --provider ollama --app-scenario support_bot\n  ai-sec sessions\n\nДополнительно:\n  ai-sec help run",
    version
)]
pub struct Cli {
    /// Выбрать конкретный провайдер: openai, anthropic, ollama, deepseek, yandexgpt
    #[arg(short, long, global = true, env = "AISEC_PROVIDER")]
    pub provider: Option<String>,

    /// Увеличить подробность вывода (-v или -vv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Запустить одну или несколько категорий атак
    #[command(
        after_help = "Примеры:\n  ai-sec run --attack prompt_injection --provider deepseek\n  ai-sec run --attack prompt_injection --provider deepseek --generated 3\n  ai-sec run --attack sensitive_data_exposure --provider ollama --app-scenario support_bot --limit 5\n  ai-sec run --attack sensitive_data_exposure --provider ollama --app-scenario internal_rag_bot --retrieval-mode subset"
    )]
    Run {
        /// ID категории атаки: jailbreaking, prompt_injection, sensitive_data_exposure
        #[arg(short, long, required = true)]
        attack: Vec<String>,

        /// Override имени модели для этого запуска
        #[arg(short, long)]
        model: Option<String>,

        /// Сохранить JSON-отчёт в указанный файл
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Ограничить число payload-ов на категорию атаки
        #[arg(short, long)]
        limit: Option<usize>,

        /// Сгенерировать до N дополнительных payload-вариантов через DeepSeek
        #[arg(long)]
        generated: Option<usize>,

        /// Сценарий приложения для scenario-driven атак, например support_bot
        #[arg(long)]
        app_scenario: Option<String>,

        /// Путь к корню synthetic fixtures
        #[arg(long)]
        fixture_root: Option<PathBuf>,

        /// Режим retrieval для scenario-driven атак: full или subset
        #[arg(long)]
        retrieval_mode: Option<String>,

        /// Явный путь к scenario manifest
        #[arg(long)]
        scenario_config: Option<PathBuf>,

        /// Tenant ID для synthetic multi-tenant сценариев
        #[arg(long)]
        tenant: Option<String>,

        /// Детерминированный seed для сборки сценария
        #[arg(long)]
        session_seed: Option<String>,
    },

    /// Показать доступные категории атак и число payload-ов
    List,

    /// Показать обучающее описание категории атаки
    Explain {
        /// ID категории атаки
        attack: String,
    },

    /// Проверить доступность и конфигурацию провайдеров
    Check,

    /// Открыть сохранённый JSON-отчёт в review-режиме
    Review {
        /// Путь к JSON-отчёту, например results/2026-04-02_14-30.json
        file: PathBuf,
    },

    /// Сравнить несколько сессий между собой
    Compare {
        /// JSON-отчёты для сравнения; если не указаны, будут загружены все файлы из results/
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },

    /// Показать обзор сохранённых сессий в results/
    Sessions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_command_parses_model_override() {
        let cli = Cli::parse_from([
            "ai-sec",
            "run",
            "--attack",
            "jailbreaking",
            "--model",
            "gpt-4.1-mini",
        ]);

        match cli.command {
            Some(Commands::Run {
                model,
                attack,
                generated,
                app_scenario,
                ..
            }) => {
                assert_eq!(attack, vec!["jailbreaking"]);
                assert_eq!(model.as_deref(), Some("gpt-4.1-mini"));
                assert_eq!(generated, None);
                assert_eq!(app_scenario, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn run_command_parses_generated_variants() {
        let cli = Cli::parse_from([
            "ai-sec",
            "run",
            "--attack",
            "prompt_injection",
            "--generated",
            "3",
        ]);

        match cli.command {
            Some(Commands::Run {
                attack,
                generated,
                app_scenario,
                ..
            }) => {
                assert_eq!(attack, vec!["prompt_injection"]);
                assert_eq!(generated, Some(3));
                assert_eq!(app_scenario, None);
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn run_command_parses_sensitive_data_flags() {
        let cli = Cli::parse_from([
            "ai-sec",
            "run",
            "--attack",
            "sensitive_data_exposure",
            "--app-scenario",
            "support_bot",
            "--retrieval-mode",
            "subset",
            "--tenant",
            "tenant-a",
            "--session-seed",
            "demo",
        ]);

        match cli.command {
            Some(Commands::Run {
                attack,
                app_scenario,
                retrieval_mode,
                tenant,
                session_seed,
                ..
            }) => {
                assert_eq!(attack, vec!["sensitive_data_exposure"]);
                assert_eq!(app_scenario.as_deref(), Some("support_bot"));
                assert_eq!(retrieval_mode.as_deref(), Some("subset"));
                assert_eq!(tenant.as_deref(), Some("tenant-a"));
                assert_eq!(session_seed.as_deref(), Some("demo"));
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }
}
