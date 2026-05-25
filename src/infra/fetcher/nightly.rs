use anyhow::Result;
use async_trait::async_trait;

use crate::domain::RustRelease;
use crate::infra::Config;

use super::http::probe_channel_history;
use super::ReleaseFetcher;

/// Nightly channel fetcher
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
    fn channel_name(&self) -> &str {
        "nightly"
    }

    fn source_description(&self) -> &str {
        "Channel TOML date probing"
    }

    async fn fetch(&self, config: &Config) -> Result<Vec<RustRelease>> {
        let releases = probe_channel_history(config, "nightly", self.days).await;
        if releases.is_empty() {
            anyhow::bail!("No nightly release data found");
        }
        Ok(releases)
    }
}
