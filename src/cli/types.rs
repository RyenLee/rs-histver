use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Rust release channel.
///
/// Represents the three Rust release tracks: stable, beta, and nightly.
#[derive(Clone, ValueEnum)]
pub enum Channel {
    Stable,
    Beta,
    Nightly,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Stable => write!(f, "stable"),
            Channel::Beta => write!(f, "beta"),
            Channel::Nightly => write!(f, "nightly"),
        }
    }
}

/// CLI definition
#[derive(Parser)]
#[command(name = "rs-histver")]
#[command(about = "Query Rust historical release versions")]
#[command(version)]
pub struct Cli {
    /// Data directory for database files
    #[arg(long, global = true)]
    pub data_dir: Option<PathBuf>,

    /// Database filename (enables shared mode when set)
    #[arg(long, global = true)]
    pub db_file: Option<String>,

    /// Prefix for database table names (default: rs_histver)
    #[arg(long, global = true)]
    pub table_prefix: Option<String>,

    /// HTTP request timeout in seconds (default: 15)
    #[arg(long, global = true)]
    pub timeout: Option<u64>,

    /// Maximum concurrent HTTP requests (default: 10)
    #[arg(long, global = true)]
    pub max_concurrency: Option<usize>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Sync release data from remote to local cache
    Sync {
        /// Release channel (stable/beta/nightly), default: stable
        #[arg(short, long, value_enum, default_value = "stable")]
        channel: Channel,

        /// Use RELEASES.md as data source (stable only, more complete)
        #[arg(long)]
        full: bool,

        /// Probe recent N days of history (beta/nightly only, default: 30)
        #[arg(short, long, default_value = "30")]
        days: u32,
    },
    /// List cached releases
    List {
        /// Maximum number of entries to display
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Filter by channel (stable/beta/nightly)
        #[arg(short, long, value_enum)]
        channel: Option<Channel>,
    },
    /// Search releases (fuzzy match by version or date)
    Search {
        /// Search keyword (e.g. "1.75" or "2024")
        keyword: String,

        /// Filter by channel (stable/beta/nightly)
        #[arg(short, long, value_enum)]
        channel: Option<Channel>,
    },
    /// Show local cache statistics
    Info {
        /// Filter by channel (stable/beta/nightly)
        #[arg(short, long, value_enum)]
        channel: Option<Channel>,
    },
}
