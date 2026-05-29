//! # rs-histver
//!
//! A library for querying Rust historical release versions.
//!
//! Pure network fetch — no database, no file system, no side effects.
//! The **CLI binary** (`rs-histver`) additionally provides terminal formatting.
//!
//! ## As a library
//!
//! ```ignore
//! use rs_histver::{fetch_releases, FetchOptions};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let releases = fetch_releases("stable", FetchOptions::default()).await?;
//!     for r in &releases {
//!         println!("{} ({})", r.version, r.date);
//!     }
//!     Ok(())
//! }
//! ```

mod constants;
mod domain;
mod options;

pub mod app;
pub mod cli;
pub(crate) mod infra;

pub use constants::*;
pub use domain::RustRelease;
pub use infra::fetcher::{create_fetcher, ReleaseFetcher};
pub use options::{FetchOptions, NetworkConfig};

use anyhow::Result;

/// Fetch Rust release data for a given channel.
///
/// # Parameters
///
/// - `channel` — `"stable"`, `"beta"`, or `"nightly"`
/// - `opts` — query configuration ([`FetchOptions`])
///
/// # Returns
///
/// `Vec<RustRelease>` sorted by date descending.
///
/// # Errors
///
/// Returns an error if the channel name is unknown or the remote request fails.
///
/// # Examples
///
/// ```ignore
/// use rs_histver::{fetch_releases, FetchOptions};
///
/// // Default options — 15s timeout, 10 concurrent, GitHub API for stable
/// let releases = fetch_releases("stable", FetchOptions::default()).await?;
///
/// // Full history from RELEASES.md
/// let releases = fetch_releases("stable",
///     FetchOptions::new().full_history(true)
/// ).await?;
///
/// // Recent 7 days of nightly builds
/// let releases = fetch_releases("nightly",
///     FetchOptions::new().probe_days(7)
/// ).await?;
/// ```
pub async fn fetch_releases(channel: &str, opts: FetchOptions) -> Result<Vec<RustRelease>> {
    let network = NetworkConfig::from(&opts);
    let fetcher = infra::fetcher::create_fetcher(channel, opts.full_history, opts.probe_days)?;
    fetcher.fetch(&network).await
}