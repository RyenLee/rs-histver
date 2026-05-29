use std::time::Duration;

use crate::constants::{
    default_user_agent, DEFAULT_MAX_CONCURRENCY, DEFAULT_PROBE_DAYS, DEFAULT_TIMEOUT_SECS,
};

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
            probe_days: DEFAULT_PROBE_DAYS,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            user_agent: default_user_agent(),
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

/// Network configuration for HTTP fetcher requests.
///
/// Created automatically from [`FetchOptions`] by [`fetch_releases`].
/// For low-level use with [`create_fetcher`], use [`NetworkConfig::new()`]
/// or construct via struct literal.
///
/// # Examples
///
/// ```ignore
/// use rs_histver::{create_fetcher, NetworkConfig, ReleaseFetcher};
///
/// let network = NetworkConfig::new()
///     .timeout_secs(30)
///     .max_concurrency(5);
/// let fetcher = create_fetcher("stable", false, 30)?;
/// let releases = fetcher.fetch(&network).await?;
/// ```
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub timeout: u64,
    pub max_concurrency: usize,
    pub user_agent: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT_SECS,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            user_agent: default_user_agent(),
        }
    }
}

impl NetworkConfig {
    /// Create a new `NetworkConfig` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the HTTP request timeout in seconds.
    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = secs;
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

impl From<&FetchOptions> for NetworkConfig {
    fn from(opts: &FetchOptions) -> Self {
        Self {
            timeout: opts.timeout.as_secs(),
            max_concurrency: opts.max_concurrency,
            user_agent: opts.user_agent.clone(),
        }
    }
}
