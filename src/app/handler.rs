#[cfg(feature = "cli")]
use anyhow::Result;

#[cfg(feature = "cli")]
use crate::cli::Commands;

#[cfg(feature = "cli")]
use crate::FetchOptions;

#[cfg(feature = "cli")]
use super::display::print_releases_table;

#[cfg(feature = "cli")]
pub struct App;

#[cfg(feature = "cli")]
impl App {
    pub async fn execute(command: Commands) -> Result<()> {
        match command {
            Commands::Fetch {
                channel,
                full,
                days,
                timeout,
            } => Self::handle_fetch(&channel.to_string(), full, days, timeout).await,
        }
    }

    async fn handle_fetch(channel: &str, full: bool, days: u32, timeout: u64) -> Result<()> {
        println!("Fetching Rust {} release data from remote...", channel);

        let opts = FetchOptions::new()
            .full_history(full)
            .probe_days(days)
            .timeout(std::time::Duration::from_secs(timeout));
        let releases = crate::fetch_releases(channel, opts).await?;

        if releases.is_empty() {
            println!("No release data found");
            return Ok(());
        }

        print_releases_table(&releases, releases.len());
        println!("\nTotal: {} releases", releases.len());
        Ok(())
    }
}