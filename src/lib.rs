//! # rs-histver
//!
//! A library for querying Rust historical release versions with local redb cache.
//!
//! ## As a CLI tool
//!
//! ```sh
//! rs-histver sync --channel stable
//! rs-histver list --limit 20
//! rs-histver search 1.75
//! ```
//!
//! ## As a library
//!
//! ```ignore
//! use rs_histver::{HistVer, Config};
//!
//! // Default: database at <exe_dir>/data/rs-histver.redb
//! let hv = HistVer::new(Config::new())?;
//!
//! // Or: custom database path
//! let hv = HistVer::new(Config::with_db_path("/path/to/custom.redb"))?;
//!
//! // Fetch and cache releases (async)
//! let releases = hv.fetch_releases("stable", false, 30).await?;
//! hv.store_releases(&releases)?;
//!
//! // Query local cache (sync)
//! let all = hv.list_releases(None)?;
//! let results = hv.search_releases("1.75", None)?;
//! let count = hv.count_releases(None)?;
//! ```

#[doc(hidden)]
pub mod app;
#[doc(hidden)]
pub mod cli;
mod domain;
pub(crate) mod infra;

// ---- Public API re-exports ----

pub use domain::RustRelease;
pub use infra::fetcher::{create_fetcher, ReleaseFetcher};
pub use infra::Config;
pub use infra::Db;

use anyhow::Result;

/// Main entry point for library usage.
///
/// Wraps `Config` and `Db`, providing a high-level API for fetching,
/// storing, and querying Rust release data.
pub struct HistVer {
    config: Config,
    db: Db,
}

impl HistVer {
    /// Create a new instance with the given config.
    ///
    /// Opens or creates the database at the path specified in config.
    /// Default database path: `<exe_dir>/data/rs-histver.redb`
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or created.
    pub fn new(config: Config) -> Result<Self> {
        let db = Db::open(&config)?;
        Ok(Self { config, db })
    }

    /// Fetch release data from remote source.
    ///
    /// - `channel`: "stable", "beta", or "nightly"
    /// - `full`: use RELEASES.md as data source (stable only)
    /// - `days`: how many days of history to probe (beta/nightly only)
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is unknown or the remote request fails.
    pub async fn fetch_releases(
        &self,
        channel: &str,
        full: bool,
        days: u32,
    ) -> Result<Vec<RustRelease>> {
        let fetcher = infra::fetcher::create_fetcher(channel, full, days)?;
        fetcher.fetch(&self.config).await
    }

    /// Store release records into the local database (upsert).
    ///
    /// Returns the number of records written.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write transaction fails.
    pub fn store_releases(&self, releases: &[RustRelease]) -> Result<u64> {
        self.db.upsert_all(releases)
    }

    /// List all cached releases, optionally filtered by channel.
    ///
    /// # Errors
    ///
    /// Returns an error if the database read fails.
    pub fn list_releases(&self, channel: Option<&str>) -> Result<Vec<RustRelease>> {
        self.db.list_all(channel)
    }

    /// Search cached releases by keyword (matches version or date).
    ///
    /// # Errors
    ///
    /// Returns an error if the database read fails.
    pub fn search_releases(
        &self,
        keyword: &str,
        channel: Option<&str>,
    ) -> Result<Vec<RustRelease>> {
        self.db.search(keyword, channel)
    }

    /// Count cached releases, optionally filtered by channel.
    ///
    /// # Errors
    ///
    /// Returns an error if the database read fails.
    pub fn count_releases(&self, channel: Option<&str>) -> Result<u64> {
        self.db.count(channel)
    }
}
