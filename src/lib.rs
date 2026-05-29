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
//! ## As a library (standalone)
//!
//! ```ignore
//! use rs_histver::{HistVer, Config};
//!
//! // Default: database at <exe_dir>/data/rs-histver.redb
//! let hv = HistVer::new(Config::new())?;
//!
//! // Or: custom database path
//! let hv = HistVer::new(Config::with_db_path("/path/to/custom.redb"))?;
//! ```
//!
//! ## As a library (embedded in host project)
//!
//! When embedded in a host project (e.g. Tauri app), the database path
//! should be determined by the host's configuration, not by the executable
//! directory. Use `ConfigBuilder` or `Config::from_config_file()` to achieve this.
//!
//! ### Standalone mode (default)
//!
//! Creates a dedicated `rs-histver.redb` file in `data_dir`.
//!
//! ```ignore
//! use rs_histver::{HistVer, ConfigBuilder};
//!
//! let hv = HistVer::new(
//!     ConfigBuilder::new()
//!         .data_dir(app_data_dir)          // creates <data_dir>/rs-histver.redb
//!         .build()?
//! )?;
//! ```
//!
//! ### Shared mode (host uses redb)
//!
//! When the host project also uses redb, you can share the same database file.
//! Configure `db_file` to point to the host's redb file, and `table_prefix`
//! to avoid table name collisions.
//!
//! The host project's `config.toml`:
//!
//! ```toml
//! [paths]
//! # Supported variable substitution:
//! #   $EXE_DIR  — executable directory
//! #   $HOME     — user home directory
//! #   ~/        — user home directory (shorthand)
//! data_dir = "$EXE_DIR/data"
//!
//! [rs-histver.database]
//! db_file = "myapp.redb"                # shared: open host's redb file
//! table_prefix = "myapp_histver"        # table names: myapp_histver_stable/beta/nightly
//!
//! [rs-histver.network]
//! timeout = 30                           # optional, default: 15 (seconds)
//! max_concurrency = 5                    # optional, default: 10
//! ```
//!
//! ```ignore
//! use rs_histver::{HistVer, Config};
//!
//! let hv = HistVer::new(Config::from_config_file("config.toml")?)?;
//! ```
//!
//! If `db_file` points to a non-redb file, it automatically falls back to
//! standalone mode (creates `rs-histver.redb` in the same directory).
//!
//! ### Programmatic configuration
//!
//! ```ignore
//! use rs_histver::{HistVer, ConfigBuilder};
//!
//! // Shared mode
//! let hv = HistVer::new(
//!     ConfigBuilder::new()
//!         .data_dir(app_data_dir)
//!         .db_file("myapp.redb")          // shared mode
//!         .table_prefix("myapp_histver")
//!         .build()?
//! )?;
//!
//! // Standalone mode
//! let hv = HistVer::new(
//!     ConfigBuilder::new()
//!         .data_dir(app_data_dir)          // standalone mode (default)
//!         .build()?
//! )?;
//! ```
//!
//! ### Priority order
//!
//! ```text
//! Programmatic override (highest) → config.toml → hardcoded defaults (lowest)
//! ```
//!
//! - `db_path()` overrides everything (complete file path, always standalone mode)
//! - `db_file()` enables shared mode, combined with `data_dir()`
//! - `data_dir()` alone → standalone mode: `<data_dir>/rs-histver.redb`
//! - `table_prefix()` / `timeout()` / `max_concurrency()` override config.toml values
//! - Missing config file or fields fall back to defaults silently
//!
//! ### Variable substitution in config.toml
//!
//! | Variable | Resolves to | Example (Windows) |
//! |----------|-------------|-------------------|
//! | `$EXE_DIR` | Executable directory | `C:\Program Files\MyApp` |
//! | `$HOME` | User home (`%USERPROFILE%` / `$HOME`) | `C:\Users\user` |
//! | `~/` | User home (same as `$HOME`) | `~/data` → `C:\Users\user\data` |
//!
//! Variable substitution only applies to paths read from config.toml.
//! Programmatic `data_dir()` / `db_path()` accept resolved `PathBuf` values.
//!
//! ## Core API
//!
//! ```ignore
//! use rs_histver::HistVer;
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
pub use infra::ConfigBuilder;
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
