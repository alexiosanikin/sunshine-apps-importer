//! A single app entry in the array format `{"apps": [{...}, {...}]}`.
//!
//! Used for serializing SunshineApps into the array format that newer
//! Sunshine versions expect.

use serde::{Deserialize, Serialize};

use crate::core::{PrepCmd, SunshineApp};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prep_cmd: Option<Vec<PrepCmd>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_global_prep_cmd: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detach: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_all: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detached: Option<Vec<String>>,
}

impl From<&SunshineApp> for AppEntry {
    fn from(app: &SunshineApp) -> Self {
        Self {
            name: app.name.clone(),
            cmd: app.cmd.clone(),
            working_directory: app.working_directory.clone(),
            output: app.output.clone(),
            image_path: app.image_path.clone(),
            poster: app.poster.clone(),
            prep_cmd: app.prep_cmd.clone(),
            elevated: app.elevated,
            exclude_global_prep_cmd: app.exclude_global_prep_cmd,
            env: app.env.clone(),
            exit_timeout: app.exit_timeout,
            auto_detach: app.auto_detach,
            wait_all: app.wait_all,
            detached: app.detached.clone(),
        }
    }
}

impl From<SunshineApp> for AppEntry {
    fn from(app: SunshineApp) -> Self {
        Self::from(&app)
    }
}
