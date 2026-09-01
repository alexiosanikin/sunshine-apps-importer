//! `version` subcommand — print version information.

use crate::i18n::I18n;

/// Handle the version command.
pub fn handle(_i18n: &'static I18n) {
    println!("Sunshine Stream Apps Manager");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Platform: {}", platform_name());
}

/// Get a human-readable platform name.
fn platform_name() -> &'static str {
    match std::env::consts::OS {
        "linux" => "Linux",
        "macos" => "macOS",
        "windows" => "Windows",
        _ => std::env::consts::OS,
    }
}
