//! A single application entry in Sunshine apps.json.
//!
//! Maps directly to the schema Sunshine expects for each game entry.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::PrepCmd;

/// A single application entry in Sunshine apps.json.
///
/// All fields except `name` are optional, matching Sunshine's schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SunshineApp {
    /// Display name shown in Sunshine.
    pub name: String,

    /// Command to launch the game (empty for Steam detached commands).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,

    /// Working directory for the game process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    /// Output capture setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Path to the game's image/icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,

    /// Path to a poster image (absolute, set before upload).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,

    /// Preparation commands (do/undo pairs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prep_cmd: Option<Vec<PrepCmd>>,

    /// Whether to run with elevated privileges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevated: Option<bool>,

    /// Exclude global prep-cmd from Sunshine config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_global_prep_cmd: Option<bool>,

    /// Environment variables to pass to the game process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    /// Timeout in seconds for game exit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_timeout: Option<i32>,

    /// Whether to auto-detach the process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detach: Option<bool>,

    /// Whether to wait for all processes to exit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_all: Option<bool>,

    /// Detached launch commands (used for Steam steam:// URLs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detached: Option<Vec<String>>,
}
