use anyhow::Result;
use async_trait::async_trait;

use crate::constants::CHANNEL_BETA;
use crate::domain::RustRelease;
use crate::options::NetworkConfig;

use super::http::probe_channel_history;
use super::ReleaseFetcher;

/// Beta channel fetcher.
///
/// Probes the static.rust-lang.org channel TOML files for the most recent
/// `days` to discover beta releases.
pub(super) struct BetaFetcher {
    days: u32,
}

impl BetaFetcher {
    pub fn new(days: u32) -> Self {
        Self { days }
    }
}

#[async_trait]
impl ReleaseFetcher for BetaFetcher {
    fn channel_name(&self) -> &'static str {
        CHANNEL_BETA
    }

    fn source_description(&self) -> &'static str {
        "Channel TOML date probing"
    }

    async fn fetch(&self, network: &NetworkConfig) -> Result<Vec<RustRelease>> {
        let releases = probe_channel_history(network, CHANNEL_BETA, self.days).await?;
        if releases.is_empty() {
            anyhow::bail!("No beta release data found");
        }
        Ok(releases)
    }
}
