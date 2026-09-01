//! Epic Games Store plugin.
//!
//! Reads `.item` manifest files from EGS launcher and scans common
//! game installation directories.

use crate::config::DefaultPaths;
use crate::core::{Game, Plugin};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ItemFile {
    #[serde(rename = "InstallLocation")]
    install_location: String,
    #[serde(rename = "FolderName")]
    folder_name: String,
    #[serde(rename = "AppName")]
    app_name: String,
    #[serde(rename = "TitleName")]
    title_name: String,
    #[serde(rename = "ExecutableFile")]
    executable_file: Option<String>,
    #[serde(rename = "LaunchCommand")]
    launch_command: Option<String>,
    #[serde(rename = "IsFullInstall")]
    is_full_install: Option<bool>,
}

pub struct EgsPlugin {
    manifest_path: Option<String>,
    install_path: Option<String>,
}

impl EgsPlugin {
    pub fn new(manifest_path: Option<String>, install_path: Option<String>) -> Self {
        Self {
            manifest_path,
            install_path,
        }
    }

    fn get_manifest_dirs(&self) -> Vec<std::path::PathBuf> {
        if let Some(ref path) = self.manifest_path {
            return vec![std::path::PathBuf::from(path)];
        }
        DefaultPaths::egs()
    }

    fn get_install_dirs(&self) -> Vec<std::path::PathBuf> {
        if let Some(ref path) = self.install_path {
            return vec![std::path::PathBuf::from(path)];
        }

        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
        let platform = std::env::consts::OS;

        match platform {
            "linux" => vec![
                home.join("Games/EpicGames"),
                home.join("Games"),
                home.join("EpicGames"),
                std::path::PathBuf::from("/mnt/gaming/EpicGames"),
            ],
            "macos" => vec![home.join("Games/EpicGames"), home.join("EpicGames")],
            "windows" => vec![
                std::path::PathBuf::from("C:\\Program Files\\Epic Games"),
                std::path::PathBuf::from("D:\\Games\\Epic Games"),
            ],
            _ => vec![home.join("Games/EpicGames")],
        }
    }

    fn scan_manifest_dir(manifest_dir: &std::path::Path) -> Result<Vec<Game>, anyhow::Error> {
        let mut games = Vec::new();

        let entries = std::fs::read_dir(manifest_dir)?;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".item") && !entry.file_type()?.is_dir() {
                if let Some(game) = Self::parse_item_file(&entry.path())? {
                    games.push(game);
                }
            }
        }

        Ok(games)
    }

    fn parse_item_file(path: &std::path::Path) -> Result<Option<Game>, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let item: ItemFile = serde_json::from_str(&content)?;

        // Skip incomplete installations
        if item.is_full_install == Some(false) {
            return Ok(None);
        }

        let game_name = if !item.title_name.is_empty() {
            item.title_name.clone()
        } else if !item.app_name.is_empty() {
            item.app_name.clone()
        } else if !item.folder_name.is_empty() {
            item.folder_name.clone()
        } else {
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        let executable = if let Some(ref exe_file) = item.executable_file {
            if !item.install_location.is_empty() {
                std::path::Path::new(&item.install_location)
                    .join(exe_file)
                    .to_string_lossy()
                    .to_string()
            } else {
                exe_file.clone()
            }
        } else if let Some(ref launch_cmd) = item.launch_command {
            if !item.install_location.is_empty() {
                std::path::Path::new(&item.install_location)
                    .join(launch_cmd)
                    .to_string_lossy()
                    .to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let working_dir = if !item.install_location.is_empty() {
            item.install_location
        } else {
            path.parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        let app_id = if !item.app_name.is_empty() {
            item.app_name.clone()
        } else {
            game_name.to_lowercase().replace(' ', "-")
        };

        Ok(Some(Game {
            id: app_id,
            name: game_name,
            executable,
            working_dir,
            icon_path: None,
            platform: "egs".to_string(),
            steam_id: None,
        }))
    }

    fn scan_install_dir(install_dir: &std::path::Path) -> Result<Vec<Game>, anyhow::Error> {
        let mut games = Vec::new();

        let entries = std::fs::read_dir(install_dir)?;
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let game_dir = entry.path();
                if Self::is_game_directory(&game_dir) {
                    let game_name = entry.file_name().to_string_lossy().to_string();
                    let executable = Self::find_executable(&game_dir);

                    games.push(Game {
                        id: game_name.to_lowercase().replace(' ', "-"),
                        name: game_name,
                        executable,
                        working_dir: game_dir.to_string_lossy().to_string(),
                        icon_path: None,
                        platform: "egs".to_string(),
                        steam_id: None,
                    });
                }
            }
        }

        Ok(games)
    }

    fn is_game_directory(dir: &std::path::Path) -> bool {
        let checks = ["Binaries", "Content", "Engine", "Saved"];
        for check in &checks {
            if dir.join(check).exists() {
                return true;
            }
        }

        // Check for Unreal Engine specific files
        let unreal_files = ["GameUserSettings.ini", "Game.ini", "Engine.ini"];
        for f in &unreal_files {
            if dir.join(f).exists() {
                return true;
            }
        }

        false
    }

    fn find_executable(dir: &std::path::Path) -> String {
        let platform = std::env::consts::OS;

        let patterns = match platform {
            "linux" => vec![
                dir.join("Binaries/Linux/*.sh"),
                dir.join("Binaries/Linux/*.so"),
            ],
            "macos" => vec![dir.join("Mac/*.app")],
            "windows" => vec![dir.join("**/*.exe")],
            _ => vec![],
        };

        for pattern in patterns {
            if let Ok(matches) = glob::glob(&pattern.to_string_lossy()) {
                for match_path in matches.flatten() {
                    if match_path.is_file() {
                        return match_path.to_string_lossy().to_string();
                    }
                }
            }
        }

        // Fallback: look for any executable in Binaries folder
        let binaries = dir.join("Binaries");
        if binaries.exists() {
            if let Ok(entries) = std::fs::read_dir(&binaries) {
                for entry in entries.flatten() {
                    if !entry.file_type().map(|t| t.is_dir()).unwrap_or(true) {
                        return entry.path().to_string_lossy().to_string();
                    }
                }
            }
        }

        dir.to_string_lossy().to_string()
    }
}

impl Plugin for EgsPlugin {
    fn name(&self) -> &str {
        "egs"
    }

    fn description(&self) -> &str {
        "Epic Games Store game library scanner"
    }

    fn scan(&self) -> Result<Vec<Game>, anyhow::Error> {
        let mut all_games = Vec::new();

        // Scan manifest directories
        for manifest_dir in self.get_manifest_dirs() {
            if !manifest_dir.exists() {
                continue;
            }
            let dir_games = Self::scan_manifest_dir(&manifest_dir)?;
            all_games.extend(dir_games);
        }

        // Scan common game install directories
        for install_dir in self.get_install_dirs() {
            if !install_dir.exists() {
                continue;
            }
            let dir_games = Self::scan_install_dir(&install_dir)?;
            all_games.extend(dir_games);
        }

        Ok(deduplicate_games(&all_games))
    }

    fn is_installed(&self) -> bool {
        self.get_manifest_dirs()
            .iter()
            .chain(self.get_install_dirs().iter())
            .any(|p| p.exists())
    }

    fn supported_platforms(&self) -> &[&'static str] {
        static PLATFORMS: &[&str] = &["linux", "darwin", "windows"];
        PLATFORMS
    }
}

fn deduplicate_games(games: &[Game]) -> Vec<Game> {
    let mut seen = std::collections::HashSet::new();
    games
        .iter()
        .filter(|g| seen.insert(g.id.clone()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_file_deserialize() {
        let json = r#"{
            "InstallLocation": "/home/user/Games/Fortnite",
            "FolderName": "Fortnite",
            "AppName": "Fortnite",
            "TitleName": "Fortnite",
            "ExecutableFile": "Fortnite/Binaries/Linux/Fortnite.sh",
            "IsFullInstall": true
        }"#;
        let item: ItemFile = serde_json::from_str(json).unwrap();
        assert_eq!(item.title_name, "Fortnite");
        assert_eq!(item.install_location, "/home/user/Games/Fortnite");
    }
}
