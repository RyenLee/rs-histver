use clap::{Parser, Subcommand};

use crate::constants::{
    CHANNEL_BETA, CHANNEL_NIGHTLY, CHANNEL_STABLE, DEFAULT_PROBE_DAYS, DEFAULT_TIMEOUT_SECS,
};

/// CLI entry for rs-histver.
///
/// No database, no config file — pure network fetch with terminal output.
#[derive(Parser, Debug)]
#[command(
    name = "rs-histver",
    version,
    about = "Query Rust historical release versions",
    long_about = "Fetch Rust release history from remote sources and display in terminal.\n\n\
                  Data sources:\n  \
                  stable     : GitHub Releases API (--full for RELEASES.md complete history)\n  \
                  beta       : dist/channel-rust-beta.toml date probing\n  \
                  nightly    : dist/channel-rust-nightly.toml date probing"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fetch release data from remote source (stable by default).
    Fetch {
        /// Release channel: stable, beta, or nightly.
        #[arg(
            short = 'c',
            long = "channel",
            default_value = "stable",
            verbatim_doc_comment,
            help = "Channel: stable, beta, nightly"
        )]
        channel: Channel,

        /// Use RELEASES.md full history data source (stable only).
        #[arg(
            long,
            help = "Use RELEASES.md full history instead of GitHub API (stable only)"
        )]
        full: bool,

        /// Probe recent N days of history (beta/nightly).
        #[arg(short = 'd', long = "days", default_value_t = DEFAULT_PROBE_DAYS, help = "Days to probe (beta/nightly)")]
        days: u32,

        /// HTTP request timeout in seconds.
        #[arg(short = 't', long = "timeout", default_value_t = DEFAULT_TIMEOUT_SECS, help = "HTTP request timeout in seconds")]
        timeout: u64,
    },
}

/// Valid release channels.
///
/// Implements `FromStr` for clap parsing and `Display` for error messages.
#[derive(Debug, Clone)]
pub enum Channel {
    Stable,
    Beta,
    Nightly,
}

impl std::str::FromStr for Channel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            CHANNEL_STABLE => Ok(Self::Stable),
            CHANNEL_BETA => Ok(Self::Beta),
            CHANNEL_NIGHTLY => Ok(Self::Nightly),
            _ => Err(format!(
                "unknown channel '{s}'. Valid values: stable, beta, nightly"
            )),
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Channel {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => CHANNEL_STABLE,
            Self::Beta => CHANNEL_BETA,
            Self::Nightly => CHANNEL_NIGHTLY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_channel_parsing() {
        let cli = Cli::try_parse_from(["rs-histver", "fetch", "-c", "stable"]).unwrap();
        match cli.command {
            Commands::Fetch { channel, .. } => assert_eq!(channel.as_str(), "stable"),
        }
    }

    #[test]
    fn test_channel_case_insensitive() {
        let cli = Cli::try_parse_from(["rs-histver", "fetch", "-c", "NIGHTLY"]).unwrap();
        match cli.command {
            Commands::Fetch { channel, .. } => assert_eq!(channel.as_str(), "nightly"),
        }
    }

    #[test]
    fn test_invalid_channel() {
        let result = Cli::try_parse_from(["rs-histver", "fetch", "-c", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_channel() {
        let cli = Cli::try_parse_from(["rs-histver", "fetch"]).unwrap();
        match cli.command {
            Commands::Fetch { channel, .. } => assert_eq!(channel.as_str(), "stable"),
        }
    }
}
