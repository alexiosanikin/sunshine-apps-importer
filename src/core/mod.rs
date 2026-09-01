//! Core domain types and the Plugin trait.
//!
//! This module defines the fundamental domain types (`Game`, `SunshineApp`,
//! `SunshineApps`) and the `Plugin` trait that all game library scanners
//! must implement.

mod app_entry;
mod errors;
mod game;
mod plugin;
mod prep_cmd;
mod sunshine_app;
mod sunshine_apps;

pub use app_entry::AppEntry;
pub use errors::*;
pub use game::Game;
pub use plugin::{Plugin, PluginManager};
pub use prep_cmd::PrepCmd;
pub use sunshine_app::SunshineApp;
pub use sunshine_apps::SunshineApps;
