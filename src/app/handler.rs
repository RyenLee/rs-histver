use anyhow::Result;

use crate::cli::Commands;
use crate::infra::fetcher;
use crate::infra::Config;
use crate::infra::Db;

use super::display::print_releases_table;

/// Application core business layer.
///
/// Holds config and database instances, encapsulates all business logic.
/// `main()` only parses CLI args and delegates to App methods.
pub struct App {
    config: Config,
    db: Db,
}

impl App {
    /// Initialize application: open database with given config
    pub fn init(config: Config) -> Result<Self> {
        let db = Db::open(&config)?;
        Ok(Self { config, db })
    }

    /// Dispatch subcommand to the corresponding handler method
    pub async fn execute(&self, command: Commands) -> Result<()> {
        match command {
            Commands::Sync {
                channel,
                full,
                days,
            } => self.handle_sync(&channel.to_string(), full, days).await,
            Commands::List { limit, channel } => {
                let ch = channel.as_ref().map(std::string::ToString::to_string);
                self.handle_list(ch.as_deref(), limit)
            }
            Commands::Search { keyword, channel } => {
                let ch = channel.as_ref().map(std::string::ToString::to_string);
                self.handle_search(&keyword, ch.as_deref())
            }
            Commands::Info { channel } => {
                let ch = channel.as_ref().map(std::string::ToString::to_string);
                self.handle_info(ch.as_deref())
            }
        }
    }

    // ---- Handlers ----

    async fn handle_sync(&self, channel: &str, full: bool, days: u32) -> Result<()> {
        let fetcher = fetcher::create_fetcher(channel, full, days)?;
        println!(
            "Fetching Rust {} release data from remote...",
            fetcher.channel_name()
        );
        println!("Data source: {}", fetcher.source_description());

        let releases = fetcher.fetch(&self.config).await?;

        if releases.is_empty() {
            println!("No release data found");
            return Ok(());
        }

        let count = self.db.upsert_all(&releases)?;
        println!(
            "Synced {} {} releases to local cache",
            count,
            fetcher.channel_name()
        );
        Ok(())
    }

    fn handle_list(&self, channel: Option<&str>, limit: usize) -> Result<()> {
        let releases = self.db.list_all(channel)?;
        if releases.is_empty() {
            let hint = channel.unwrap_or("any channel");
            println!(
                "Local cache is empty ({hint}), run `rs-histver sync` first"
            );
            return Ok(());
        }

        print_releases_table(&releases, limit);

        if releases.len() > limit {
            println!(
                "\nShowing top {} of {} entries (use --limit for more)",
                limit,
                releases.len()
            );
        }
        Ok(())
    }

    fn handle_search(&self, keyword: &str, channel: Option<&str>) -> Result<()> {
        let results = self.db.search(keyword, channel)?;
        if results.is_empty() {
            println!("No releases matching \"{keyword}\" found");
            return Ok(());
        }

        print_releases_table(&results, results.len());

        let ch_label = channel.unwrap_or("all channels");
        println!("\nFound {} matching results ({})", results.len(), ch_label);
        Ok(())
    }

    fn handle_info(&self, channel: Option<&str>) -> Result<()> {
        let ch_label = channel.unwrap_or("all");
        let count = self.db.count(channel)?;
        println!("Cached releases ({ch_label}): {count}");
        Ok(())
    }
}
