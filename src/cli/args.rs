//! CLI argument definitions using clap derive.

use clap::Parser;

use crate::cli::commands;

/// Sunshine Stream Apps Manager — manage Sunshine game library configuration.
///
/// Integrates with Sunshine stream manager to read game libraries from
/// multiple services (Steam, Epic Games Store, Lutris) and generate
/// proper `apps.json` configuration.
#[derive(Parser, Debug)]
#[command(name = "sunshine-stream-apps-manager")]
#[command(about = "Manage Sunshine stream apps configuration")]
pub struct CliArgs {
    /// Config file path (default: config.yaml next to binary or in config dir)
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Generate config but do not write it
    #[arg(long, global = true, default_value_t = false)]
    pub dry_run: bool,

    /// Language (en, ru, fr, it, es, el, ka)
    #[arg(long, global = true, default_value = "")]
    pub language: String,

    /// Enable verbose output
    #[arg(long, global = true, default_value_t = false)]
    pub verbose: bool,

    /// Show default paths for each service and exit
    #[arg(long, global = true, default_value_t = false)]
    pub show_paths: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: commands::Command,
}
