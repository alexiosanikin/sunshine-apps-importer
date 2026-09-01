//! Lutris game library plugin.
//!
//! Reads `games.yml` from Lutris configuration directory and parses
//! YAML game entries.

use crate::core::{Game, Plugin};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[expect(dead_code)]
struct LutrisGame {
    name: String,
    slug: String,
    directory: Option<String>,
    exe: Option<String>,
    runner: Option<String>,
    installed: Option<i32>,
    guid: Option<String>,
}

impl LutrisGame {
    fn is_installed(&self) -> bool {
        self.installed == Some(1)
    }
}

pub struct LutrisPlugin {
    custom_path: Option<String>,
}

impl LutrisPlugin {
    pub fn new(custom_path: Option<String>) -> Self {
        Self { custom_path }
    }

    fn get_games_dir(&self) -> Option<std::path::PathBuf> {
        if let Some(ref path) = self.custom_path {
            return Some(std::path::PathBuf::from(path));
        }

        dirs::home_dir().map(|home| home.join(".local/share/lutris"))
    }

    fn find_executable(dir: &str) -> String {
        if dir.is_empty() {
            return String::new();
        }

        let candidates = [
            "game.sh",
            "Game.sh",
            "run.sh",
            "Run.sh",
            "start.sh",
            "Start.sh",
            "launch.sh",
            "Launch.sh",
            "play.sh",
            "Play.sh",
            "game",
            "Game",
        ];

        for name in &candidates {
            let path = std::path::Path::new(dir).join(name);
            if path.exists() {
                return path.to_string_lossy().to_string();
            }
        }

        dir.to_string()
    }
}

impl Plugin for LutrisPlugin {
    fn name(&self) -> &str {
        "lutris"
    }

    fn description(&self) -> &str {
        "Lutris game library scanner"
    }

    fn scan(&self) -> Result<Vec<Game>, anyhow::Error> {
        let games_dir = self.get_games_dir();
        let games_dir = match games_dir {
            Some(p) if p.exists() => p,
            _ => return Ok(Vec::new()),
        };

        let games_file = games_dir.join("games.yml");
        if !games_file.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&games_file)?;
        let lutris_games: Vec<LutrisGame> = serde_yaml::from_str(&content)?;

        let mut games = Vec::new();
        for lg in lutris_games {
            if !lg.is_installed() {
                continue;
            }
            if lg.name.is_empty() {
                continue;
            }

            let executable = if let Some(ref exe) = lg.exe {
                exe.clone()
            } else {
                lg.directory
                    .as_ref()
                    .map(|d| Self::find_executable(d))
                    .unwrap_or_default()
            };

            let working_dir = lg.directory.unwrap_or_else(|| "/tmp".to_string());

            let app_id = if !lg.slug.is_empty() {
                lg.slug
            } else {
                lg.name.to_lowercase().replace(' ', "-")
            };

            games.push(Game {
                id: app_id,
                name: lg.name,
                executable,
                working_dir,
                icon_path: None,
                platform: "lutris".to_string(),
                steam_id: None,
            });
        }

        Ok(deduplicate_games(&games))
    }

    fn is_installed(&self) -> bool {
        if let Some(ref path) = self.custom_path {
            return std::path::Path::new(path).exists();
        }
        dirs::home_dir()
            .map(|home| home.join(".local/share/lutris").exists())
            .unwrap_or(false)
    }

    fn supported_platforms(&self) -> &[&'static str] {
        static PLATFORMS: &[&str] = &["linux", "darwin"];
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
    fn test_lutris_game_deserialize() {
        let yaml = r#"
- name: Test Game
  slug: test-game
  directory: /home/user/Games/test-game
  exe: /home/user/Games/test-game/run.sh
  runner: linux
  installed: 1
"#;
        let games: Vec<LutrisGame> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Test Game");
        assert!(games[0].is_installed());
    }
}
