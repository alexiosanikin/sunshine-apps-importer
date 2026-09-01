//! Game library plugins.

pub mod egs;
pub mod lutris;
pub mod steam;

pub use egs::EgsPlugin;
pub use lutris::LutrisPlugin;
pub use steam::SteamPlugin;
