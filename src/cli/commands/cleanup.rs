//! `cleanup` subcommand — remove tool-managed apps.

use clap::Parser;
use std::path::PathBuf;
use tracing::info;

use crate::config::AppConfig;
use crate::i18n::I18n;

/// Arguments for the cleanup command.
#[derive(Parser, Debug)]
pub struct CleanupArgs {
    /// Plugins to clean up (comma-separated). Default: all.
    #[arg(long)]
    pub plugins: Option<String>,
}

/// Handle the cleanup command.
pub async fn handle(
    config: &AppConfig,
    args: CleanupArgs,
    _i18n: &'static I18n,
) -> Result<(), anyhow::Error> {
    // Resolve Sunshine path
    let (sunshine_path_opt, _transport) = config.sunshine_path();
    let sunshine_path = match sunshine_path_opt {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            let mut found = None;
            for path in crate::config::DefaultPaths::sunshine() {
                if path.exists() {
                    found = Some(path);
                    break;
                }
            }
            found.unwrap_or_else(|| {
                eprintln!(
                    "Could not find Sunshine config directory. Set sunshine_config_dir in config."
                );
                std::process::exit(1);
            })
        }
    };

    let apps_json_path = sunshine_path.join("apps.json");

    // Determine plugins to clean
    let plugins = args
        .plugins
        .as_ref()
        .map(|s| s.split(',').map(String::from).collect::<Vec<_>>())
        .unwrap_or_else(|| config.plugins.clone());

    let plugins = if plugins.is_empty() {
        vec!["steam".to_string(), "egs".to_string()]
    } else {
        plugins
    };

    // Load existing apps
    info!("Loading existing Sunshine apps...");
    let mut existing_apps = crate::core::SunshineApps::load(&apps_json_path)?;
    info!("Loaded {} existing apps", existing_apps.count());

    // Remove plugin apps
    let removed = existing_apps.remove_plugin_apps(&plugins);

    if removed.is_empty() {
        println!("No tool-managed apps found. Nothing to clean up.");
        return Ok(());
    }

    println!("Found {} app(s) to remove:", removed.len());
    for id in &removed {
        println!("  - {id}");
    }

    // Dry run check
    if config.dry_run {
        println!("\n=== DRY RUN MODE - No changes written ===");
        println!(
            "Would write {} apps to: {}",
            existing_apps.count(),
            apps_json_path.display()
        );
        return Ok(());
    }

    // Create backup
    if apps_json_path.exists() {
        println!("Creating backup...");
        let suffix = config.backup_suffix();
        crate::services::BackupService::create(&apps_json_path, &suffix)?;
    }

    // Write cleaned config
    println!("Writing cleaned apps.json...");
    existing_apps.save(&apps_json_path)?;

    println!(
        "Cleanup complete! Removed {} app(s), {} remaining.",
        removed.len(),
        existing_apps.count()
    );

    Ok(())
}
