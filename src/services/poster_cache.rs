//! Local cache for downloaded poster images.
//!
//! Manages a `.cached/posters/` directory for storing downloaded game
//! artwork, allowing lookups by SteamID across runs.

use std::path::{Path, PathBuf};

/// Local cache for downloaded poster images.
pub struct PosterCache {
    cache_dir: PathBuf,
}

impl PosterCache {
    /// Create a new poster cache rooted at the given project directory.
    #[must_use]
    pub fn new(project_root: &str) -> Self {
        Self {
            cache_dir: PathBuf::from(project_root).join(".cached").join("posters"),
        }
    }

    /// Ensure the cache directory exists.
    pub fn ensure_dir(&self) -> Result<(), anyhow::Error> {
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }

    /// Get the cached poster path for a SteamID, or `None` if not cached.
    #[must_use]
    pub fn get_poster_path(&self, steam_id: &str) -> Option<PathBuf> {
        let path = self.cache_dir.join(format!("{steam_id}.png"));
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Cache a downloaded poster with a SteamID-based name.
    ///
    /// If the poster is already cached, returns the existing path.
    pub fn cache_poster(
        &self,
        local_path: &Path,
        steam_id: &str,
    ) -> Result<PathBuf, anyhow::Error> {
        let data = std::fs::read(local_path)?;
        let cached_path = self.cache_dir.join(format!("{steam_id}.png"));

        if cached_path.exists() {
            return Ok(cached_path);
        }

        std::fs::write(&cached_path, data)?;
        Ok(cached_path)
    }

    /// Check if a poster is cached for the given SteamID.
    #[must_use]
    pub fn is_cached(&self, steam_id: &str) -> bool {
        self.get_poster_path(steam_id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir_creation() {
        let cache = PosterCache::new("/tmp/ssam_test_cache");
        assert!(cache.cache_dir.ends_with(".cached/posters"));
    }
}
