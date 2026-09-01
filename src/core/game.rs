//! A discovered game from any library source.

use serde::{Deserialize, Serialize};

/// Represents a discovered game from any library source (Steam, EGS, Lutris).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Game {
    /// Unique identifier for the game.
    pub id: String,

    /// Human-readable display name.
    pub name: String,

    /// Path to the game executable.
    pub executable: String,

    /// Working directory for the game.
    pub working_dir: String,

    /// Optional path to a cached icon/poster image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,

    /// Library source platform (steam, egs, lutris).
    pub platform: String,

    /// Optional Steam App ID for metadata lookups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steam_id: Option<String>,
}
