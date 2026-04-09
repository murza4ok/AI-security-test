//! ai-sec entry point.

mod app;
mod attacks;
mod cli;
mod config;
mod education;
mod engine;
mod generator;
mod payloads;
mod providers;
mod reporting;
mod scenarios;

use anyhow::{Context, Result};
use clap::Parser;
use cli::args::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    let app_config =
        config::AppConfig::from_env().context("Failed to load configuration from environment")?;

    app::run(cli, app_config).await
}
