use anyhow::Result;

use crate::cli::Commands;
use crate::FetchOptions;

use super::display::print_releases_table;

/// Application core business layer.
///
/// Holds no state — stateless dispatch to `fetch_releases()`.
/// `main()` parses CLI args and delegates to handler methods.
pub struct App;

impl App {
    /// Dispatch subcommand to the corresponding handler method.
    pub async fn execute(command: Commands) -> Result<()> {
        match command {
            Commands::Fetch {
                channel,
                full,
                days,
            } => Self::handle_fetch(&channel.to_string(), full, days).await,
        }
    }

    async fn handle_fetch(channel: &str, full: bool, days: u32) -> Result<()> {
        println!("Fetching Rust {} release data from remote...", channel);

        let opts = FetchOptions::new()
            .full_history(full)
            .probe_days(days);
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