//! CLI subcommand definitions.

pub mod cleanup;
pub mod generate;
pub mod scan;
pub mod show_paths;
pub mod version;

use clap::Subcommand;

/// CLI subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Scan for installed games from various sources.
    Scan(scan::ScanArgs),

    /// Generate Sunshine config with discovered games.
    Generate(generate::GenerateArgs),

    /// Remove all tool-managed apps from Sunshine config.
    Cleanup(cleanup::CleanupArgs),

    /// Print the version number.
    Version,

    /// Show default paths for each service.
    #[command(name = "show-paths")]
    ShowPaths,
}
