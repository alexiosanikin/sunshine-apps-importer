//! `scan` subcommand — scan for installed games.

use clap::Parser;

use crate::config::AppConfig;
use crate::core::{Plugin, PluginManager};
use crate::i18n::I18n;
use crate::plugins::{EgsPlugin, LutrisPlugin, SteamPlugin};

/// Arguments for the scan command.
#[derive(Parser, Debug)]
pub struct ScanArgs {
    /// Output format: text, json
    #[arg(short = 'f', long, default_value = "text")]
    pub format: String,

    /// Plugins to scan (comma-separated, e.g., steam,egs)
    #[arg(long)]
    pub plugins: Option<String>,

    /// Scan source: local, ssh
    #[arg(long, default_value = "local")]
    pub source: String,

    /// SSH host for remote scanning
    #[arg(long)]
    pub ssh_host: Option<String>,

    /// SSH port for remote scanning
    #[arg(long, default_value = "22")]
    pub ssh_port: i32,

    /// SSH user for remote scanning
    #[arg(long)]
    pub ssh_user: Option<String>,

    /// SSH key path for remote scanning
    #[arg(long)]
    pub ssh_key: Option<String>,

    /// Verbose output
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

/// Handle the scan command.
pub async fn handle(
    config: &AppConfig,
    args: ScanArgs,
    _i18n: &'static I18n,
) -> Result<(), anyhow::Error> {
    // Determine enabled plugins
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

    // Create plugin manager and register plugins
    let mut manager = PluginManager::new();

    // Steam plugin
    let steam_path = config.steam_path().0;
    let steam_plugin = SteamPlugin::new(steam_path);
    manager.register(Box::new(steam_plugin))?;

    // EGS plugin
    let (egs_manifest, _) = config.egs_manifest_path();
    let (egs_install, _) = config.egs_install_path();
    let egs_plugin = EgsPlugin::new(egs_manifest, egs_install);
    manager.register(Box::new(egs_plugin))?;

    // Lutris plugin (Linux/macOS only)
    let lutris_path = config.lutris_path().0;
    let lutris_plugin = LutrisPlugin::new(lutris_path);
    if lutris_plugin.is_installed() {
        manager.register(Box::new(lutris_plugin))?;
    }

    // Scan
    let games = manager.scan_enabled(&plugins)?;

    if games.is_empty() {
        println!("No games found.");
        return Ok(());
    }

    // Output results
    match args.format.as_str() {
        "json" => output_json(&games),
        _ => output_text(&games, args.verbose),
    }

    Ok(())
}

fn output_text(games: &[crate::core::Game], verbose: bool) {
    println!("Found {} game(s)\n", games.len());

    // Group by platform
    let mut platforms: std::collections::HashMap<&str, Vec<&crate::core::Game>> =
        std::collections::HashMap::new();
    for game in games {
        platforms.entry(&game.platform).or_default().push(game);
    }

    for (platform, platform_games) in &platforms {
        println!(
            "[{}] ({} games)",
            platform.to_uppercase(),
            platform_games.len()
        );
        for game in platform_games {
            println!("  {}", game.name);
            if verbose {
                println!("    Executable: {}", game.executable);
                println!("    Working Dir: {}", game.working_dir);
                if let Some(ref steam_id) = game.steam_id {
                    println!("    Steam ID: {steam_id}");
                }
            }
        }
        println!();
    }
}

fn output_json(games: &[crate::core::Game]) {
    println!("[");
    for (i, game) in games.iter().enumerate() {
        let comma = if i < games.len() - 1 { "," } else { "" };
        println!(
            r#"  {{"id": "{}", "name": "{}", "executable": "{}", "working_dir": "{}", "platform": "{}", "steam_id": "{}"}}{}"#,
            game.id,
            game.name,
            game.executable,
            game.working_dir,
            game.platform,
            game.steam_id.as_deref().unwrap_or(""),
            comma
        );
    }
    println!("]");
}
