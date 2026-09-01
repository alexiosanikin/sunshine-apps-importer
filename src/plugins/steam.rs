//! Steam game library plugin.
//!
//! Reads `appmanifest_<appid>.acf` files from Steam library folders,
//! parses VDF format to extract game name, executable, and working directory.

use crate::config::DefaultPaths;
use crate::core::{Game, Plugin};

pub struct SteamPlugin {
    custom_path: Option<String>,
}

impl SteamPlugin {
    pub fn new(custom_path: Option<String>) -> Self {
        Self { custom_path }
    }

    fn get_steam_dirs(&self) -> Vec<std::path::PathBuf> {
        if let Some(ref path) = self.custom_path {
            return vec![std::path::PathBuf::from(expand_path(path))];
        }
        DefaultPaths::steam()
    }

    fn get_library_folders(steam_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut folders = Vec::new();
        let library_file = steam_dir.join("steamapps").join("libraryfolders.vdf");

        if !library_file.exists() {
            folders.push(steam_dir.join("steamapps"));
            return folders;
        }

        if let Ok(content) = std::fs::read_to_string(&library_file) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains(r#""path""#) {
                    if let Some(path) = extract_vdf_value(trimmed) {
                        folders.push(std::path::PathBuf::from(path).join("steamapps"));
                    }
                }
            }
        }

        if folders.is_empty() {
            folders.push(steam_dir.join("steamapps"));
        }

        folders
    }

    fn scan_library_folder(folder: &std::path::Path) -> Result<Vec<Game>, anyhow::Error> {
        let mut games = Vec::new();

        let entries = std::fs::read_dir(folder)?;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("appmanifest_")
                && name_str.ends_with(".acf")
                && !entry.file_type()?.is_dir()
            {
                if let Some(game) = Self::parse_manifest(&entry.path())? {
                    games.push(game);
                }
            }
        }

        Ok(games)
    }

    fn parse_manifest(path: &std::path::Path) -> Result<Option<Game>, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let app_id = extract_app_id(path);

        let name = extract_vdf_value_from_content(&content, r#""name""#);
        let app_state = extract_vdf_value_from_content(&content, r#""AppState""#);

        // AppState == 131745 means fully installed
        let is_installed = app_state == "131745" || app_state.is_empty();
        if !is_installed || name.is_empty() {
            return Ok(None);
        }

        let executable = extract_vdf_value_from_content(&content, r#""exe""#);
        let full_executable =
            if !executable.is_empty() && !std::path::Path::new(&executable).is_absolute() {
                let game_dir = path.parent().unwrap_or_else(|| std::path::Path::new(""));
                std::path::Path::new(game_dir)
                    .join(executable)
                    .to_string_lossy()
                    .to_string()
            } else {
                executable
            };

        let working_dir = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(Some(Game {
            id: app_id.clone(),
            name,
            executable: full_executable,
            working_dir,
            icon_path: None,
            platform: "steam".to_string(),
            steam_id: Some(app_id),
        }))
    }
}

impl Plugin for SteamPlugin {
    fn name(&self) -> &str {
        "steam"
    }

    fn description(&self) -> &str {
        "Steam game library scanner"
    }

    fn scan(&self) -> Result<Vec<Game>, anyhow::Error> {
        let mut all_games = Vec::new();

        for steam_dir in self.get_steam_dirs() {
            if !steam_dir.exists() {
                continue;
            }

            let library_folders = Self::get_library_folders(&steam_dir);
            for folder in library_folders {
                if !folder.exists() {
                    continue;
                }
                let folder_games = Self::scan_library_folder(&folder)?;
                all_games.extend(folder_games);
            }
        }

        Ok(deduplicate_games(&all_games))
    }

    fn is_installed(&self) -> bool {
        self.get_steam_dirs().iter().any(|p| p.exists())
    }

    fn supported_platforms(&self) -> &[&'static str] {
        static PLATFORMS: &[&str] = &["linux", "darwin", "windows"];
        PLATFORMS
    }
}

fn extract_app_id(path: &std::path::Path) -> String {
    let base = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let base = base.trim_start_matches("appmanifest_");

    base.chars().take_while(|c| c.is_ascii_digit()).collect()
}

fn extract_vdf_value(line: &str) -> Option<String> {
    // Extract value after "path" "value" pattern
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

fn extract_vdf_value_from_content(content: &str, key: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains(key) {
            if let Some(val) = extract_vdf_value(trimmed) {
                return val;
            }
        }
    }
    String::new()
}

fn deduplicate_games(games: &[Game]) -> Vec<Game> {
    let mut seen = std::collections::HashSet::new();
    games
        .iter()
        .filter(|g| seen.insert(g.id.clone()))
        .cloned()
        .collect()
}

fn expand_path(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped).to_string_lossy().to_string();
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_app_id() {
        let path = std::path::Path::new("/some/path/appmanifest_730.acf");
        assert_eq!(extract_app_id(path), "730");

        let path = std::path::Path::new("/some/path/appmanifest_570.acf");
        assert_eq!(extract_app_id(path), "570");
    }

    #[test]
    fn test_expand_path() {
        let expanded = expand_path("~/test/path");
        if let Some(home) = dirs::home_dir() {
            assert!(expanded.starts_with(home.as_path().to_string_lossy().as_ref()));
        }
    }
}
