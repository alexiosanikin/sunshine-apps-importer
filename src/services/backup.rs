//! Backup service for config files.
//!
//! Creates timestamped backups of configuration files before modifications.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Service for creating backups of config files.
pub struct BackupService;

impl BackupService {
    /// Create a backup of a file with a timestamp suffix.
    ///
    /// If the file doesn't exist, returns the original path (no error).
    ///
    /// # Errors
    /// Returns an error if reading or writing the file fails.
    pub fn create(path: &Path, suffix: &str) -> Result<PathBuf, anyhow::Error> {
        if !path.exists() {
            return Ok(path.to_path_buf());
        }

        let data = std::fs::read(path)?;
        let backup_path = PathBuf::from(format!("{}.{}", path.display(), suffix));
        std::fs::write(&backup_path, data)?;
        Ok(backup_path)
    }

    /// Generate a timestamp suffix for backups.
    ///
    /// Format: `YYYY-MM-DD_HH-MM-SS`
    #[must_use]
    pub fn timestamp_suffix() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs();
        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
            .unwrap_or_else(chrono::Utc::now);
        dt.format("%Y-%m-%d_%H-%M-%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_suffix_format() {
        let suffix = BackupService::timestamp_suffix();
        // Should match YYYY-MM-DD_HH-MM-SS
        assert_eq!(suffix.len(), 19);
        assert_eq!(suffix.chars().nth(4).unwrap(), '-');
        assert_eq!(suffix.chars().nth(7).unwrap(), '-');
        assert_eq!(suffix.chars().nth(10).unwrap(), '_');
        assert_eq!(suffix.chars().nth(13).unwrap(), '-');
        assert_eq!(suffix.chars().nth(16).unwrap(), '-');
    }

    #[test]
    fn test_backup_nonexistent_file() {
        let path = PathBuf::from("/tmp/nonexistent_file_12345.json");
        let result = BackupService::create(&path, "backup");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), path);
    }
}
