//! App merging logic.
//!
//! Handles merging discovered games into existing Sunshine configuration
//! while preserving manually-added apps.

use crate::core::{Game, SunshineApps};

/// Merging strategy configuration.
pub struct AppMerger {
    /// Whether to keep existing non-plugin apps.
    keep_existing: bool,

    /// Whether to overwrite existing plugin apps.
    overwrite: bool,
}

impl AppMerger {
    /// Create a new app merger.
    #[must_use]
    pub const fn new(keep_existing: bool, overwrite: bool) -> Self {
        Self {
            keep_existing,
            overwrite,
        }
    }

    /// Merge discovered games into existing apps.
    ///
    /// # Strategy
    /// 1. Start with non-plugin apps (if `keep_existing`)
    /// 2. For each game:
    ///    - Resolve the target app ID
    ///    - If `overwrite` is false and the app exists, skip it
    ///    - If `overwrite` is true, add the new app
    ///    - Fall back to name matching if ID-based lookup fails
    /// 3. Return the merged result with add/skip counts
    pub fn merge(
        &self,
        existing_apps: &SunshineApps,
        games: &[Game],
        plugin_names: &[String],
        use_flat_spawn: bool,
    ) -> MergeResult {
        let mut merged_apps = SunshineApps::new();

        // Start with non-plugin apps to keep
        if self.keep_existing {
            let non_plugin_ids = existing_apps.get_non_plugin_apps(plugin_names);
            for app_id in non_plugin_ids {
                if let Some(app) = existing_apps.get(&app_id) {
                    merged_apps.apps.insert(app_id, app.clone());
                }
            }
        }

        let mut added = 0usize;
        let mut skipped = 0usize;

        for game in games {
            let app_id = self.resolve_app_id(&game.id, plugin_names);

            // Check by ID match
            if let Some(existing_app) = existing_apps.get(&app_id) {
                if !self.overwrite {
                    let mut app = existing_app.clone();
                    if let Some(ref icon_path) = game.icon_path {
                        app.poster = Some(icon_path.clone());
                    }
                    merged_apps.apps.insert(app_id.clone(), app);
                    skipped += 1;
                    continue;
                }
            }

            // Check by name match as fallback
            let name_match = existing_apps
                .apps
                .iter()
                .find(|(_, app)| app.name == game.name);

            if let Some((name_match_id, name_match_app)) = name_match {
                if !self.overwrite {
                    let mut app = name_match_app.clone();
                    if let Some(ref icon_path) = game.icon_path {
                        app.poster = Some(icon_path.clone());
                    }
                    merged_apps.apps.insert(name_match_id.clone(), app);
                    skipped += 1;
                    continue;
                }
            }

            merged_apps.add_game(game, use_flat_spawn);
            added += 1;
        }

        MergeResult {
            apps: merged_apps,
            added,
            skipped,
        }
    }

    /// Resolve the target app ID for a game.
    ///
    /// Prepends the plugin name as a prefix if the game's platform
    /// matches a known plugin.
    fn resolve_app_id(&self, game_id: &str, plugin_names: &[String]) -> String {
        let plugin_prefix = plugin_names
            .iter()
            .find(|p| game_id == **p || game_id.starts_with(&format!("{p}_")))
            .map(|p| format!("{p}_"));

        let clean: String = game_id
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => c,
                _ => '_',
            })
            .collect();

        if let Some(prefix) = plugin_prefix {
            format!("{prefix}{clean}")
        } else {
            clean
        }
    }
}

/// Result of a merge operation.
pub struct MergeResult {
    /// The merged apps collection.
    pub apps: SunshineApps,

    /// Number of new apps added.
    pub added: usize,

    /// Number of existing apps skipped.
    pub skipped: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Game;

    #[test]
    fn test_merge_result_counts() {
        let merger = AppMerger::new(true, true);
        let existing = SunshineApps::new();
        let games: Vec<Game> = vec![Game {
            id: "730".to_string(),
            name: "Counter-Strike 2".to_string(),
            executable: "".to_string(),
            working_dir: "".to_string(),
            icon_path: None,
            platform: "steam".to_string(),
            steam_id: Some("730".to_string()),
        }];

        let result = merger.merge(&existing, &games, &["steam".to_string()], false);
        assert_eq!(result.added, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.apps.count(), 1);
    }

    #[test]
    fn test_merge_with_existing() {
        let merger = AppMerger::new(true, false);
        let mut existing = SunshineApps::new();
        existing.add_game(
            &Game {
                id: "730".to_string(),
                name: "Counter-Strike 2".to_string(),
                executable: "".to_string(),
                working_dir: "".to_string(),
                icon_path: None,
                platform: "steam".to_string(),
                steam_id: Some("730".to_string()),
            },
            false,
        );

        let games: Vec<Game> = vec![Game {
            id: "730".to_string(),
            name: "Counter-Strike 2".to_string(),
            executable: "".to_string(),
            working_dir: "".to_string(),
            icon_path: None,
            platform: "steam".to_string(),
            steam_id: Some("730".to_string()),
        }];

        let result = merger.merge(&existing, &games, &["steam".to_string()], false);
        assert_eq!(result.added, 0);
        assert_eq!(result.skipped, 1);
    }
}
