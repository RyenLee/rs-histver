#[cfg(feature = "cli")]
mod config;
#[cfg(feature = "cli")]
mod database;

pub mod fetcher;

#[cfg(feature = "cli")]
pub use config::Config;
#[cfg(feature = "cli")]
pub use config::ConfigBuilder;
#[cfg(feature = "cli")]
pub use database::Db;
