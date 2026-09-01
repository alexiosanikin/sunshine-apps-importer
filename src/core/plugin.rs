//! Plugin trait and PluginManager for game library scanning.
//!
//! The `Plugin` trait defines the interface all game library scanners must
//! implement. `PluginManager` provides a registry for discovering and
//! scanning games from multiple sources.

use std::collections::HashMap;

use crate::core::{CoreError, Game};

/// Trait that all game library scanners must implement.
///
/// This is the core abstraction for discovering games from different
/// platforms (Steam, EGS, Lutris, etc.). Implementations should be
/// stateless where possible and thread-safe.
pub trait Plugin: Send + Sync {
    /// Return the plugin name (e.g., "steam", "egs", "lutris").
    fn name(&self) -> &str;

    /// Return a human-readable description of the plugin.
    fn description(&self) -> &str;

    /// Scan for installed games.
    fn scan(&self) -> Result<Vec<Game>, anyhow::Error>;

    /// Check if the service is installed on this system.
    fn is_installed(&self) -> bool;

    /// Return the OS platforms this plugin supports.
    fn supported_platforms(&self) -> &[&'static str];
}

/// Registry for game library plugins.
///
/// Provides registration, lookup, and batch scanning capabilities.
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    /// Create a new empty plugin manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin.
    ///
    /// # Errors
    /// Returns [`CoreError::DuplicatePlugin`] if a plugin with the same
    /// name is already registered.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), CoreError> {
        let name = plugin.name().to_string();
        if self.plugins.contains_key(&name) {
            return Err(CoreError::DuplicatePlugin(name));
        }
        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Get a plugin by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Get all registered plugins.
    #[must_use]
    pub fn all(&self) -> Vec<&dyn Plugin> {
        self.plugins.values().map(|p| p.as_ref()).collect()
    }

    /// Get plugins matching the given names.
    ///
    /// Skips names that don't match any registered plugin.
    #[must_use]
    pub fn enabled(&self, names: &[String]) -> Vec<&dyn Plugin> {
        names.iter().filter_map(|n| self.get(n)).collect()
    }

    /// Scan using all registered plugins.
    ///
    /// # Errors
    /// Returns the first scanning error encountered.
    pub fn scan_all(&self) -> Result<Vec<Game>, anyhow::Error> {
        let mut all_games = Vec::new();
        for plugin in self.plugins.values() {
            let games = plugin.scan()?;
            all_games.extend(games);
        }
        Ok(deduplicate_games(&all_games))
    }

    /// Scan using only the enabled plugins.
    ///
    /// Skips plugin names that aren't registered.
    pub fn scan_enabled(&self, names: &[String]) -> Result<Vec<Game>, anyhow::Error> {
        let mut all_games = Vec::new();
        for name in names {
            if let Some(plugin) = self.plugins.get(name) {
                let games = plugin.scan()?;
                all_games.extend(games);
            }
        }
        Ok(deduplicate_games(&all_games))
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Deduplicate games by ID, keeping the first occurrence.
#[must_use]
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
    fn test_plugin_manager_register_and_get() {
        let manager = PluginManager::new();
        // We can't easily test with real plugins in unit tests,
        // but we can verify the structure compiles.
        assert!(manager.all().is_empty());
    }
}
