//! Service layer: AppMerger, BackupService, PosterCache.
//!
//! Implements the business logic for merging discovered games into Sunshine
//! configuration, creating backups, and managing poster images.

mod app_merger;
mod backup;
mod poster_cache;
mod posters;

pub use app_merger::{AppMerger, MergeResult};
pub use backup::BackupService;
pub use poster_cache::PosterCache;
pub use posters::save_posters;
