use anyhow::Result;
use async_trait::async_trait;

use crate::domain::RustRelease;
use crate::infra::Config;

use super::http::probe_channel_history;
use super::ReleaseFetcher;

/// Beta channel fetcher
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
    fn channel_name(&self) -> &str {
        "beta"
    }

    fn source_description(&self) -> &str {
        "Channel TOML date probing"
    }

    async fn fetch(&self, config: &Config) -> Result<Vec<RustRelease>> {
        let releases = probe_channel_history(config, "beta", self.days).await;
        if releases.is_empty() {
            anyhow::bail!("No beta release data found");
        }
        Ok(releases)
    }
}
