//! Poster image management.
//!
//! Handles copying/downloading poster images to the Sunshine posters
//! directory and returns app_id -> relative path mappings.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::SunshineApps;

/// Save poster images from apps to a local posters directory.
///
/// Only processes apps with absolute `poster` paths. Returns a mapping
/// of app_id -> relative poster path for callers to apply after saving JSON.
///
/// # Errors
/// Returns an error if reading or writing any poster file fails.
pub fn save_posters(
    apps: &SunshineApps,
    posters_dir: &Path,
) -> Result<HashMap<String, String>, anyhow::Error> {
    let posters_path = posters_dir.join("posters");
    std::fs::create_dir_all(&posters_path)?;

    let mut poster_paths = HashMap::new();

    for (app_id, app) in &apps.apps {
        let poster = match &app.poster {
            Some(p) if p.starts_with('/') => p.clone(),
            _ => continue,
        };

        let ext = PathBuf::from(&poster)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_else(|| ".png".to_string());

        let poster_dest = posters_path.join(format!("{app_id}{ext}"));
        let data = std::fs::read(&poster)?;
        std::fs::write(&poster_dest, data)?;

        let abs_path = posters_dir.join("posters").join(format!("{app_id}{ext}"));
        poster_paths.insert(app_id.clone(), abs_path.to_string_lossy().to_string());
    }

    Ok(poster_paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posters_dir_creation() {
        let dir = PathBuf::from("/tmp/ssam_posters_test");
        let result = save_posters(&SunshineApps::new(), &dir);
        assert!(result.is_ok());
        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
