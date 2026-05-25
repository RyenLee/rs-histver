use serde::{Deserialize, Serialize};

/// Rust release information.
///
/// Core data model shared by database, fetcher, and app modules.
/// No dependency on any other business module to avoid circular dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustRelease {
    /// Release version string (e.g. "1.75.0", "1.76.0-nightly")
    pub version: String,
    /// Release date in YYYY-MM-DD format
    pub date: String,
    /// Release channel: "stable", "beta", or "nightly"
    pub channel: String,
}

impl RustRelease {
    /// Generate redb primary key: "channel:version:date"
    pub fn db_key(&self) -> String {
        format!("{}:{}:{}", self.channel, self.version, self.date)
    }
}
