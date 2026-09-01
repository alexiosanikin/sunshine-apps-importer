//! Sunshine Stream Apps Manager — main entry point.
//!
//! Integrates with Sunshine stream manager to read game libraries from
//! multiple services (Steam, Epic Games Store, Lutris) and generate
//! proper `apps.json` configuration.

pub mod cli;
pub mod config;
pub mod core;
pub mod i18n;
pub mod metadata;
pub mod plugins;
pub mod services;
pub mod ssh;
