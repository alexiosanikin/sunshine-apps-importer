//! `generate` subcommand — generate Sunshine apps.json.

use clap::Parser;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::core::{Plugin, PluginManager, SunshineApps};
use crate::i18n::I18n;
use crate::plugins::{EgsPlugin, LutrisPlugin, SteamPlugin};
use crate::services::{AppMerger, BackupService, PosterCache};

/// Arguments for the generate command.
#[derive(Parser, Debug)]
pub struct GenerateArgs {
    /// Keep existing non-plugin apps
    #[arg(long, default_value_t = true)]
    pub keep_existing: bool,

    /// Overwrite existing plugin apps
    #[arg(long, default_value_t = true)]
    pub overwrite: bool,

    /// Fetch metadata from SteamGridDB
    #[arg(long, default_value_t = false)]
    pub fetch_metadata: bool,

    /// SteamGridDB API key
    #[arg(long)]
    pub steamgrid_key: Option<String>,
}

/// Handle the generate command.
pub async fn handle(
    config: &AppConfig,
    args: GenerateArgs,
    _i18n: &'static I18n,
) -> Result<(), anyhow::Error> {
    // Determine enabled plugins
    let plugins = if config.plugins.is_empty() {
        vec!["steam".to_string(), "egs".to_string()]
    } else {
        config.plugins.clone()
    };

    // Override SteamGrid key from flag
    let steamgrid_key = args
        .steamgrid_key
        .clone()
        .or_else(|| config.steamgrid_api_key.clone());

    let fetch_metadata = args.fetch_metadata || config.fetch_metadata;

    // Resolve Sunshine path
    let (sunshine_path_opt, _transport) = config.sunshine_path();
    let sunshine_path = match sunshine_path_opt {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            // Try to find in default locations
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

    info!("Loading existing Sunshine apps...");
    info!("Apps JSON path: {}", apps_json_path.display());

    // Load existing apps.json
    let existing_apps = SunshineApps::load(&apps_json_path)?;
    info!("Loaded {} existing apps", existing_apps.count());

    // Create plugin manager and scan
    let mut manager = PluginManager::new();

    let steam_path = config.steam_path().0;
    manager.register(Box::new(SteamPlugin::new(steam_path)))?;

    let (egs_manifest, _) = config.egs_manifest_path();
    let (egs_install, _) = config.egs_install_path();
    manager.register(Box::new(EgsPlugin::new(egs_manifest, egs_install)))?;

    let lutris_path = config.lutris_path().0;
    let lutris_plugin = LutrisPlugin::new(lutris_path);
    if lutris_plugin.is_installed() {
        manager.register(Box::new(lutris_plugin))?;
    }

    info!("Scanning for games...");
    let games = manager.scan_enabled(&plugins)?;

    if games.is_empty() {
        println!("No games found. Nothing to generate.");
        return Ok(());
    }

    info!("Found {} games", games.len());

    // Setup poster cache
    let poster_cache = PosterCache::new(".");
    if let Err(e) = poster_cache.ensure_dir() {
        warn!("Could not create poster cache directory: {e}");
    }

    // Fetch metadata if requested
    let games = if let Some(ref key) = steamgrid_key {
        if fetch_metadata {
            fetch_metadata_step(&games, key, &poster_cache).await?
        } else {
            games
        }
    } else {
        games
    };

    // Merge games into existing apps
    info!("Merging games into config...");
    let merger = AppMerger::new(args.keep_existing, args.overwrite);
    let mut merge_result = merger.merge(&existing_apps, &games, &plugins, config.use_flat_spawn);

    info!(
        "Added: {}, Skipped: {}",
        merge_result.added, merge_result.skipped
    );

    // Save posters
    if fetch_metadata && steamgrid_key.is_some() {
        if let Ok(poster_paths) = crate::services::save_posters(&merge_result.apps, &sunshine_path)
        {
            // Apply image paths
            for (app_id, rel_path) in &poster_paths {
                if let Some(app) = merge_result.apps.apps.get_mut(app_id) {
                    app.image_path = Some(rel_path.clone());
                }
            }
            info!("Saved {} posters", poster_paths.len());
        }
    }

    // Save generated config to cache
    let cached_dir = PathBuf::from(".cached");
    std::fs::create_dir_all(&cached_dir)?;
    let cached_path = cached_dir.join("apps_generated.json");
    merge_result.apps.save(&cached_path)?;
    info!(
        "Cached config: {} ({} apps)",
        cached_path.display(),
        merge_result.apps.count()
    );

    // Dry run check
    if config.dry_run {
        println!("\n=== DRY RUN MODE ===");
        println!(
            "  Would write {} apps to: {}",
            merge_result.apps.count(),
            apps_json_path.display()
        );
        println!("  Added: {} games", merge_result.added);
        return Ok(());
    }

    // Create backup
    if apps_json_path.exists() {
        println!("Creating backup...");
        let suffix = config.backup_suffix();
        BackupService::create(&apps_json_path, &suffix)?;
    }

    // Write apps.json
    println!("Writing apps.json...");
    merge_result.apps.save(&apps_json_path)?;

    println!("Configuration updated successfully!");
    println!("Total apps: {}", merge_result.apps.count());
    println!("Added: {} games", merge_result.added);

    Ok(())
}

/// Fetch metadata from SteamGridDB and cache posters.
async fn fetch_metadata_step(
    games: &[crate::core::Game],
    api_key: &str,
    poster_cache: &PosterCache,
) -> Result<Vec<crate::core::Game>, anyhow::Error> {
    use crate::metadata::Client as MetadataClient;

    println!("Fetching metadata from SteamGridDB...");
    let client = MetadataClient::new(api_key);
    let mut fetched = 0usize;
    let mut skipped = 0usize;

    let mut games = games.to_vec();

    for game in games.iter_mut() {
        let Some(ref steam_id) = game.steam_id else {
            skipped += 1;
            continue;
        };

        // Skip if already has an icon
        if game.icon_path.is_some() {
            fetched += 1;
            continue;
        }

        // Try to parse SteamID
        let steam_id_num: i32 = steam_id.parse().unwrap_or(0);
        if steam_id_num == 0 {
            skipped += 1;
            continue;
        }

        // Check cache
        if let Some(cached_path) = poster_cache.get_poster_path(steam_id) {
            game.icon_path = Some(cached_path.to_string_lossy().to_string());
            fetched += 1;
            continue;
        }

        // Download poster
        let tmp_dir = std::env::temp_dir().join(format!("ssam-posters-{}", steam_id));
        std::fs::create_dir_all(&tmp_dir)?;
        let poster_path = tmp_dir.join(format!("{steam_id_num}.png"));

        if let Ok(Some(grid_url)) = client.get_best_grid_url(steam_id_num).await {
            if client
                .download_grid(&grid_url, poster_path.to_string_lossy().as_ref())
                .await
                .is_ok()
            {
                if let Ok(cached) = poster_cache.cache_poster(&poster_path, steam_id) {
                    game.icon_path = Some(cached.to_string_lossy().to_string());
                }
                fetched += 1;
            }
        }

        // Cleanup temp dir
        let _ = std::fs::remove_dir_all(&tmp_dir);

        // Rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    tracing::info!("Metadata: fetched={fetched}, skipped={skipped}");
    Ok(games)
}
