mod beta;
mod http;
mod nightly;
mod stable;

use crate::domain::RustRelease;
use crate::infra::Config;
use anyhow::Result;
use async_trait::async_trait;

/// Release fetcher strategy trait (Strategy Pattern).
///
/// Each Rust release channel implements this trait with its own remote data
/// source and parsing logic.
#[async_trait]
pub trait ReleaseFetcher: Send + Sync {
    /// Human-readable channel name (e.g. "stable", "beta", "nightly").
    fn channel_name(&self) -> &str;
    /// Human-readable description of the data source being used.
    fn source_description(&self) -> &str;
    /// Fetch release data from the remote source.
    async fn fetch(&self, config: &Config) -> Result<Vec<RustRelease>>;
}

/// Factory function: create the appropriate fetcher for a given channel.
///
/// - `channel`: one of `"stable"`, `"beta"`, or `"nightly"`.
/// - `full`: if `true`, stable fetcher uses RELEASES.md full history instead of GitHub API.
/// - `days`: how many days of history to probe (beta/nightly only).
///
/// # Errors
///
/// Returns an error if the channel name is not recognized.
pub fn create_fetcher(channel: &str, full: bool, days: u32) -> Result<Box<dyn ReleaseFetcher>> {
    match channel {
        "stable" => Ok(Box::new(stable::StableFetcher::new(full))),
        "beta" => Ok(Box::new(beta::BetaFetcher::new(days))),
        "nightly" => Ok(Box::new(nightly::NightlyFetcher::new(days))),
        _ => anyhow::bail!("Unknown channel: '{channel}'. Must be one of: stable, beta, nightly"),
    }
}
