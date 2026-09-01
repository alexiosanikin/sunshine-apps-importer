//! A preparation command for Sunshine.
//!
//! Sunshine uses prep-cmd to run commands before launching a game and
//! commands to undo them when the game exits.

use serde::{Deserialize, Serialize};

/// A preparation command pair (do/undo) for Sunshine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrepCmd {
    /// Command to run before launching the game.
    #[serde(rename = "do")]
    pub do_cmd: String,

    /// Command to run after the game exits.
    #[serde(rename = "undo")]
    pub undo_cmd: String,

    /// Whether to run with elevated privileges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevated: Option<bool>,
}
