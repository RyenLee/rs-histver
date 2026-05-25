mod beta;
mod http;
mod nightly;
mod stable;

use crate::domain::RustRelease;
use crate::infra::Config;
use anyhow::Result;
use async_trait::async_trait;

/// Release fetcher strategy trait (Strategy Pattern)
#[async_trait]
pub trait ReleaseFetcher: Send + Sync {
    fn channel_name(&self) -> &str;
    fn source_description(&self) -> &str;
    async fn fetch(&self, config: &Config) -> Result<Vec<RustRelease>>;
}

/// Factory function: create a fetcher for the given channel (Factory Method Pattern)
pub fn create_fetcher(channel: &str, full: bool, days: u32) -> Box<dyn ReleaseFetcher> {
    match channel {
        "stable" => Box::new(stable::StableFetcher::new(full)),
        "beta" => Box::new(beta::BetaFetcher::new(days)),
        "nightly" => Box::new(nightly::NightlyFetcher::new(days)),
        _ => unreachable!("Unknown channel: {}", channel),
    }
}
