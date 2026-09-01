//! CLI entry point and command routing.
//!
//! This module provides the main entry point for the application,
//! parsing CLI arguments and routing to the appropriate command handler.

mod args;
mod commands;

use clap::Parser;

pub use args::CliArgs;

/// Main CLI entry point.
///
/// Parses command-line arguments and routes to the appropriate
/// command handler (scan, generate, cleanup, version).
pub async fn run() -> Result<(), anyhow::Error> {
    let args = CliArgs::parse();

    // Initialize i18n
    let i18n = crate::i18n::get_i18n();
    i18n.set_language(&args.language)?;

    // Load configuration
    let config = load_config(&args)?;

    // Route to command handler
    match args.command {
        commands::Command::Scan(scan_args) => {
            commands::scan::handle(&config, scan_args, i18n).await
        }
        commands::Command::Generate(generate_args) => {
            commands::generate::handle(&config, generate_args, i18n).await
        }
        commands::Command::Cleanup(cleanup_args) => {
            commands::cleanup::handle(&config, cleanup_args, i18n).await
        }
        commands::Command::Version => {
            commands::version::handle(i18n);
            Ok(())
        }
        commands::Command::ShowPaths => {
            commands::show_paths::handle();
            Ok(())
        }
    }
}

/// Load configuration from file or environment.
fn load_config(args: &CliArgs) -> Result<crate::config::AppConfig, anyhow::Error> {
    let mut config = crate::config::AppConfig::load()?;

    // Apply CLI flag overrides
    if args.dry_run {
        config.dry_run = true;
    }
    if !args.language.is_empty() {
        config.language = args.language.clone();
    }

    Ok(config)
}
