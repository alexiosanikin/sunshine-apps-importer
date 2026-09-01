//! Collection of Sunshine applications with loading, saving, and management.
//!
//! Supports both the object format `{"app_id": {...}}` and the array format
//! `{"apps": [{...}, {...}]}` used by newer Sunshine versions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::core::{AppEntry, CoreError, Game, PrepCmd, SunshineApp};

/// Collection of all Sunshine applications.
///
/// Internally stores apps in a `HashMap` keyed by sanitized app ID for
/// efficient lookups, but supports both JSON formats Sunshine uses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunshineApps {
    #[serde(flatten)]
    #[serde(skip_serializing)]
    pub apps: HashMap<String, SunshineApp>,
}

impl SunshineApps {
    /// Create an empty collection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            apps: HashMap::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Loading / Saving
    // -------------------------------------------------------------------------

    /// Load apps.json from a file path.
    ///
    /// Returns an empty collection if the file doesn't exist (non-error).
    pub fn load(path: &Path) -> Result<Self, CoreError> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let data = std::fs::read_to_string(path)?;
        let parsed: serde_json::Value = serde_json::from_str(&data)?;

        // Try object format first: {"app_id": {...}}
        if let Some(obj) = parsed.as_object() {
            let mut apps = HashMap::new();
            for (key, value) in obj {
                if let Ok(app) = serde_json::from_value(value.clone()) {
                    apps.insert(key.clone(), app);
                }
            }
            if !apps.is_empty() {
                return Ok(Self { apps });
            }
        }

        // Try array format: {"apps": [{...}, {...}]}
        if let Some(wrapper) = parsed.as_object() {
            if let Some(apps_array) = wrapper.get("apps").and_then(|v| v.as_array()) {
                return Self::from_array(apps_array);
            }
        }

        Err(CoreError::InvalidFormat)
    }

    /// Save apps to a JSON file in the array format.
    pub fn save(&self, path: &Path) -> Result<(), CoreError> {
        let dir = path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(dir)?;

        let apps_array: Vec<AppEntry> = self.apps.values().map(AppEntry::from).collect();
        let wrapper = serde_json::json!({ "apps": apps_array });
        let data = serde_json::to_string_pretty(&wrapper)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    fn from_array(apps_array: &[serde_json::Value]) -> Result<Self, CoreError> {
        let mut apps = HashMap::new();

        for (i, entry_value) in apps_array.iter().enumerate() {
            let app: SunshineApp = serde_json::from_value(entry_value.clone())?;
            let app_id = sanitize_app_id(&app.name);
            let app_id = if app_id.is_empty() {
                format!("app_{i}")
            } else {
                let mut unique_id = app_id;
                let mut counter = 1usize;
                while apps.contains_key(&unique_id) {
                    unique_id = format!("{}_{}_{}", sanitize_app_id(&app.name), "app", counter);
                    counter += 1;
                }
                unique_id
            };
            apps.insert(app_id, app);
        }

        Ok(Self { apps })
    }

    // -------------------------------------------------------------------------
    // Game management
    // -------------------------------------------------------------------------

    /// Add a game to the apps collection.
    ///
    /// Steam games get special handling with detached `steam://` commands.
    pub fn add_game(&mut self, game: &Game, use_flat_spawn: bool) {
        let app_id = sanitize_app_id(&game.id);
        if app_id.is_empty() {
            return;
        }

        let mut app = SunshineApp {
            name: game.name.clone(),
            cmd: None,
            working_directory: non_empty(&game.working_dir),
            output: None,
            image_path: None,
            poster: game.icon_path.clone(),
            prep_cmd: None,
            elevated: Some(false),
            exclude_global_prep_cmd: Some(false),
            env: None,
            exit_timeout: Some(5),
            auto_detach: Some(true),
            wait_all: Some(true),
            detached: None,
        };

        // Steam games use detached command with steam://run/<appid>
        if game.platform == "steam" {
            if let Some(ref steam_id) = game.steam_id {
                let launch_cmd = if use_flat_spawn {
                    format!("flatpak-spawn --host setsid steam steam://run/{steam_id}")
                } else {
                    format!("steam steam://run/{steam_id}")
                };
                let close_cmd = if use_flat_spawn {
                    format!("flatpak-spawn --host setsid steam steam://close/{steam_id}")
                } else {
                    format!("steam steam://close/{steam_id}")
                };

                app.prep_cmd = Some(vec![PrepCmd {
                    do_cmd: String::new(),
                    undo_cmd: close_cmd,
                    elevated: Some(false),
                }]);
                app.detached = Some(vec![launch_cmd]);
            }
        }

        self.apps.insert(app_id, app);
    }

    // -------------------------------------------------------------------------
    // Querying
    // -------------------------------------------------------------------------

    /// Get an app by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&SunshineApp> {
        self.apps.get(id)
    }

    /// Get the total number of apps.
    #[must_use]
    pub fn count(&self) -> usize {
        self.apps.len()
    }

    /// Get app IDs that belong to specific plugin names.
    ///
    /// Matches IDs with prefixes like `steam_`, `egs_`, `lutris_`.
    #[must_use]
    pub fn get_plugin_apps(&self, plugin_names: &[String]) -> Vec<String> {
        let prefixes: Vec<String> = plugin_names.iter().map(|p| format!("{p}_")).collect();

        self.apps
            .keys()
            .filter(|id| prefixes.iter().any(|p| id.starts_with(p)))
            .cloned()
            .collect()
    }

    /// Get app IDs that do NOT belong to the given plugin names.
    #[must_use]
    pub fn get_non_plugin_apps(&self, plugin_names: &[String]) -> Vec<String> {
        let prefixes: Vec<String> = plugin_names.iter().map(|p| format!("{p}_")).collect();

        self.apps
            .keys()
            .filter(|id| !prefixes.iter().any(|p| id.starts_with(p)))
            .cloned()
            .collect()
    }

    /// Remove all apps belonging to the given plugin names.
    ///
    /// In addition to prefix matching, also detects Steam games by their
    /// `steam://run/` and `steam://close/` patterns in prep-cmd.
    pub fn remove_plugin_apps(&mut self, plugin_names: &[String]) -> Vec<String> {
        let prefixes: Vec<String> = plugin_names.iter().map(|p| format!("{p}_")).collect();
        let steam_run_re = regex::Regex::new(r"(?i)steam://run/\d+").unwrap();
        let steam_close_re = regex::Regex::new(r"(?i)steam://close/\d+").unwrap();

        let mut removed = Vec::new();
        let keys: Vec<String> = self.apps.keys().cloned().collect();

        for id in keys {
            let should_remove = if let Some(app) = self.apps.get(&id) {
                let by_prefix = prefixes.iter().any(|p| id.starts_with(p));

                let by_prep_cmd = app
                    .prep_cmd
                    .as_ref()
                    .map(|cmds| {
                        cmds.first().is_some_and(|cmd| {
                            steam_run_re.is_match(&cmd.do_cmd)
                                && steam_close_re.is_match(&cmd.undo_cmd)
                        })
                    })
                    .unwrap_or(false);

                let by_cmd = app
                    .cmd
                    .as_ref()
                    .map(|cmd| steam_run_re.is_match(cmd))
                    .unwrap_or(false);

                by_prefix || by_prep_cmd || by_cmd
            } else {
                false
            };

            if should_remove {
                self.apps.remove(&id);
                removed.push(id);
            }
        }

        removed
    }

    /// Remove an app by ID.
    pub fn remove(&mut self, id: &str) {
        self.apps.remove(id);
    }
}

impl Default for SunshineApps {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitize an ID to be a valid JSON key (alphanumeric + underscore only).
#[must_use]
pub fn sanitize_app_id(id: &str) -> String {
    id.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => c,
            _ => '_',
        })
        .collect()
}

#[inline]
#[must_use]
fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_app_id() {
        assert_eq!(sanitize_app_id("730"), "730");
        assert_eq!(sanitize_app_id("my-game"), "my_game");
        assert_eq!(sanitize_app_id("Fortnite"), "Fortnite");
        assert_eq!(sanitize_app_id("Half-Life 3"), "Half_Life_3");
    }

    #[test]
    fn test_sunshine_apps_new() {
        let apps = SunshineApps::new();
        assert_eq!(apps.count(), 0);
    }

    #[test]
    fn test_sunshine_apps_add_game() {
        let mut apps = SunshineApps::new();
        let game = Game {
            id: "730".to_string(),
            name: "Counter-Strike 2".to_string(),
            executable: "".to_string(),
            working_dir: "".to_string(),
            icon_path: None,
            platform: "steam".to_string(),
            steam_id: Some("730".to_string()),
        };
        apps.add_game(&game, false);
        assert_eq!(apps.count(), 1);
        assert!(apps.get("730").is_some());

        let app = apps.get("730").unwrap();
        assert_eq!(app.name, "Counter-Strike 2");
        assert!(app.detached.is_some());
        assert!(app.prep_cmd.is_some());
    }

    #[test]
    fn test_remove_plugin_apps() {
        let mut apps = SunshineApps::new();

        // Add a Steam game with "steam_" prefix (as the generator would)
        apps.apps.insert(
            "steam_730".to_string(),
            SunshineApp {
                name: "CS2".to_string(),
                cmd: None,
                working_directory: None,
                output: None,
                image_path: None,
                poster: None,
                prep_cmd: Some(vec![PrepCmd {
                    do_cmd: "steam steam://run/730".to_string(),
                    undo_cmd: "steam steam://close/730".to_string(),
                    elevated: Some(false),
                }]),
                elevated: Some(false),
                exclude_global_prep_cmd: Some(false),
                env: None,
                exit_timeout: Some(5),
                auto_detach: Some(true),
                wait_all: Some(true),
                detached: Some(vec!["steam steam://run/730".to_string()]),
            },
        );

        // Add an EGS game
        apps.add_game(
            &Game {
                id: "egs-fortnite".to_string(),
                name: "Fortnite".to_string(),
                executable: "".to_string(),
                working_dir: "".to_string(),
                icon_path: None,
                platform: "egs".to_string(),
                steam_id: None,
            },
            false,
        );

        let removed = apps.remove_plugin_apps(&["steam".to_string()]);
        assert_eq!(removed.len(), 1);
        assert_eq!(apps.count(), 1);
    }
}
