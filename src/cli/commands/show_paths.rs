//! `show-paths` subcommand — display default paths for each service.

use crate::config::DefaultPaths;

/// Handle the show-paths command.
pub fn handle() {
    let platform = platform_name();
    println!("Platform: {platform}\n");

    println!("Steam paths:");
    for path in DefaultPaths::steam() {
        let status = if path.exists() { "found" } else { "not found" };
        println!("  {} [{status}]", path.display());
    }

    println!("\nEpic Games Store paths:");
    for path in DefaultPaths::egs() {
        let status = if path.exists() { "found" } else { "not found" };
        println!("  {} [{status}]", path.display());
    }

    println!("\nSunshine paths:");
    for path in DefaultPaths::sunshine() {
        let status = if path.exists() { "found" } else { "not found" };
        println!("  {} [{status}]", path.display());
    }

    println!("\nLutris paths:");
    for path in DefaultPaths::lutris() {
        let status = if path.exists() { "found" } else { "not found" };
        println!("  {} [{status}]", path.display());
    }
}

fn platform_name() -> &'static str {
    match std::env::consts::OS {
        "linux" => "Linux",
        "macos" => "macOS",
        "windows" => "Windows",
        _ => std::env::consts::OS,
    }
}
