use std::time::Duration;

/// Options for fetching Rust release data.
///
/// All fields have sensible defaults, so `FetchOptions::default()` or
/// `FetchOptions::new()` gives a reasonable starting configuration.
///
/// # Examples
///
/// ```ignore
/// use rs_histver::{fetch_releases, FetchOptions};
///
/// // Default options
/// let releases = fetch_releases("stable", FetchOptions::default()).await?;
///
/// // Full history from RELEASES.md
/// let releases = fetch_releases("stable",
///     FetchOptions::new().full_history(true)
/// ).await?;
///
/// // Probe recent 7 days of nightly releases
/// let releases = fetch_releases("nightly",
///     FetchOptions::new().probe_days(7)
/// ).await?;
/// ```
#[derive(Debug, Clone)]
pub struct FetchOptions {
    /// Stable channel: use RELEASES.md for full historical data (default: false = GitHub API)
    pub full_history: bool,
    /// Beta/Nightly channels: probe recent N days (default: 30)
    pub probe_days: u32,
    /// HTTP request timeout (default: 15 seconds)
    pub timeout: Duration,
    /// Maximum concurrent HTTP requests (default: 10)
    pub max_concurrency: usize,
    /// User-Agent header string
    pub user_agent: String,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            full_history: false,
            probe_days: 30,
            timeout: Duration::from_secs(15),
            max_concurrency: 10,
            user_agent: format!("rs-histver/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl FetchOptions {
    /// Create a new `FetchOptions` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable RELEASES.md full history mode (stable only).
    #[must_use]
    pub fn full_history(mut self, v: bool) -> Self {
        self.full_history = v;
        self
    }

    /// Set the number of days to probe for beta/nightly channels.
    #[must_use]
    pub fn probe_days(mut self, days: u32) -> Self {
        self.probe_days = days;
        self
    }

    /// Set the HTTP request timeout.
    #[must_use]
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = d;
        self
    }

    /// Set the maximum number of concurrent HTTP requests.
    #[must_use]
    pub fn max_concurrency(mut self, n: usize) -> Self {
        self.max_concurrency = n;
        self
    }

    /// Set a custom User-Agent header.
    #[must_use]
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }
}

/// Internal network configuration, derived from [`FetchOptions`].
///
/// Used by fetcher implementations.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub timeout: u64,
    pub max_concurrency: usize,
    pub user_agent: String,
}

impl From<&FetchOptions> for NetworkConfig {
    fn from(opts: &FetchOptions) -> Self {
        Self {
            timeout: opts.timeout.as_secs(),
            max_concurrency: opts.max_concurrency,
            user_agent: opts.user_agent.clone(),
        }
    }
}