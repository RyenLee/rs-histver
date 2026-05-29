use anyhow::Result;
use async_trait::async_trait;

use crate::constants::CHANNEL_NIGHTLY;
use crate::domain::RustRelease;
use crate::options::NetworkConfig;

use super::http::probe_channel_history;
use super::ReleaseFetcher;

/// Nightly channel fetcher.
///
/// Probes the static.rust-lang.org channel TOML files for the most recent
/// `days` to discover nightly releases.
pub(super) struct NightlyFetcher {
    days: u32,
}

impl NightlyFetcher {
    pub fn new(days: u32) -> Self {
        Self { days }
    }
}

#[async_trait]
impl ReleaseFetcher for NightlyFetcher {
    fn channel_name(&self) -> &'static str {
        CHANNEL_NIGHTLY
    }

    fn source_description(&self) -> &'static str {
        "Channel TOML date probing"
    }

    async fn fetch(&self, network: &NetworkConfig) -> Result<Vec<RustRelease>> {
        let releases = probe_channel_history(network, CHANNEL_NIGHTLY, self.days).await?;
        if releases.is_empty() {
            anyhow::bail!("No nightly release data found");
        }
        Ok(releases)
    }
}
