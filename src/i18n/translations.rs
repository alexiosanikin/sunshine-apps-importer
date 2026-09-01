//! Default translation data.
//!
//! Provides English translations as a fallback when the JSON files
//! aren't available at runtime.

use std::collections::HashMap;

/// Default English translations.
///
/// These are used as a fallback if the translation JSON file
/// can't be loaded from disk.
#[must_use]
#[expect(dead_code)]
pub fn english_translations() -> HashMap<String, String> {
    HashMap::from([
        // App info
        (
            "app.name".to_string(),
            "Sunshine Stream Apps Manager".to_string(),
        ),
        ("app.version".to_string(), "Version".to_string()),
        ("app.commit".to_string(), "Commit".to_string()),
        ("app.built".to_string(), "Built".to_string()),
        ("app.platform".to_string(), "Platform".to_string()),
        // Commands
        (
            "cmd.scan".to_string(),
            "Scan for installed games".to_string(),
        ),
        (
            "cmd.generate".to_string(),
            "Generate Sunshine config".to_string(),
        ),
        (
            "cmd.cleanup".to_string(),
            "Remove tool-managed apps".to_string(),
        ),
        ("cmd.version".to_string(), "Print version info".to_string()),
        ("cmd.config".to_string(), "Manage configuration".to_string()),
        ("cmd.paths".to_string(), "Show default paths".to_string()),
        // Scan output
        (
            "cmd.scan.games_found".to_string(),
            "Found {} game(s)".to_string(),
        ),
        (
            "cmd.scan.no_games".to_string(),
            "No games found.".to_string(),
        ),
        // Generate output
        (
            "cmd.generate.loading".to_string(),
            "Loading existing Sunshine apps...".to_string(),
        ),
        (
            "cmd.generate.scanning".to_string(),
            "Scanning for games...".to_string(),
        ),
        (
            "cmd.generate.found".to_string(),
            "Found {} game(s)".to_string(),
        ),
        (
            "cmd.generate.merging".to_string(),
            "Merging games into config...".to_string(),
        ),
        (
            "cmd.generate.backup".to_string(),
            "Creating backup...".to_string(),
        ),
        (
            "cmd.generate.writing".to_string(),
            "Writing apps.json...".to_string(),
        ),
        (
            "cmd.generate.success".to_string(),
            "Configuration updated successfully!".to_string(),
        ),
        (
            "cmd.generate.dry_run".to_string(),
            "=== DRY RUN MODE ===".to_string(),
        ),
        // Cleanup output
        (
            "cmd.cleanup.removing".to_string(),
            "Removing tool-managed apps...".to_string(),
        ),
        (
            "cmd.cleanup.nothing".to_string(),
            "No tool-managed apps found. Nothing to clean up.".to_string(),
        ),
        (
            "cmd.cleanup.complete".to_string(),
            "Cleanup complete!".to_string(),
        ),
        // Errors
        (
            "error.config_load".to_string(),
            "Failed to load config: {0}".to_string(),
        ),
        (
            "error.scan_failed".to_string(),
            "Scan failed: {0}".to_string(),
        ),
        (
            "error.merge_failed".to_string(),
            "Merge failed: {0}".to_string(),
        ),
        (
            "error.save_failed".to_string(),
            "Failed to save: {0}".to_string(),
        ),
        (
            "error.backup_failed".to_string(),
            "Failed to create backup: {0}".to_string(),
        ),
        // Plugin info
        (
            "plugin.steam".to_string(),
            "Steam game library scanner".to_string(),
        ),
        (
            "plugin.egs".to_string(),
            "Epic Games Store game library scanner".to_string(),
        ),
        (
            "plugin.lutris".to_string(),
            "Lutris game library scanner".to_string(),
        ),
        // Paths
        ("path.steam".to_string(), "Steam paths:".to_string()),
        (
            "path.egs".to_string(),
            "Epic Games Store paths:".to_string(),
        ),
        ("path.sunshine".to_string(), "Sunshine paths:".to_string()),
        ("path.lutris".to_string(), "Lutris paths:".to_string()),
    ])
}
