//! Centralized constants for the `rs-histver` project.
//!
//! All channel names, default configuration values, URL endpoints, and magic
//! numbers are defined here. Import via `use crate::constants::*;` for
//! convenience, or reference individually.

// ── Channel name constants ──────────────────────────────────────────────

/// Stable release channel identifier.
pub const CHANNEL_STABLE: &str = "stable";
/// Beta release channel identifier.
pub const CHANNEL_BETA: &str = "beta";
/// Nightly release channel identifier.
pub const CHANNEL_NIGHTLY: &str = "nightly";

/// All three valid channel names, ordered for iteration.
pub const ALL_CHANNELS: &[&str] = &[CHANNEL_STABLE, CHANNEL_BETA, CHANNEL_NIGHTLY];

// ── Default configuration values ────────────────────────────────────────

/// Default number of days to probe for beta/nightly history.
pub const DEFAULT_PROBE_DAYS: u32 = 30;

/// Default HTTP request timeout (seconds).
pub const DEFAULT_TIMEOUT_SECS: u64 = 15;

/// Default maximum number of concurrent HTTP requests.
pub const DEFAULT_MAX_CONCURRENCY: usize = 10;

// ── GitHub API constants ────────────────────────────────────────────────

/// Results per page for the GitHub Releases API.
pub const GITHUB_PER_PAGE: u32 = 100;

/// Maximum number of pages to fetch from GitHub Releases API
/// (1500 releases — ~10 years of Rust history).
pub const GITHUB_MAX_PAGES: u32 = 15;

// ── Remote data source URLs ─────────────────────────────────────────────

/// GitHub Releases API endpoint for the Rust language repository.
pub const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/rust-lang/rust/releases";

/// Raw RELEASES.md for full historical stable release data.
pub const RELEASES_MD_URL: &str =
    "https://raw.githubusercontent.com/rust-lang/rust/master/RELEASES.md";

/// Base URL for static.rust-lang.org distribution files.
pub const STATIC_DIST_BASE_URL: &str = "https://static.rust-lang.org/dist";

// ── Package identity ────────────────────────────────────────────────────

/// Package / binary name, used for CLI identity and User-Agent prefix.
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// Generate the default `User-Agent` header string, e.g. `"rs-histver/0.4.0"`.
#[must_use]
pub fn default_user_agent() -> String {
    format!("{PKG_NAME}/{}", env!("CARGO_PKG_VERSION"))
}