pub mod interactive;
pub mod providers;
pub mod runtime;
pub mod scenarios;

use crate::cli::args::Cli;
use crate::config;
use anyhow::Result;

pub async fn run(cli: Cli, app_config: config::AppConfig) -> Result<()> {
    let provider_override = cli.provider.clone();
    let verbose = cli.verbose;

    match cli.command {
        None => interactive::run_interactive(cli, app_config).await,
        Some(cmd) => {
            let cmd_cli = Cli {
                provider: provider_override,
                verbose,
                command: None,
            };
            runtime::run_command(cmd, cmd_cli, app_config).await
        }
    }
}
