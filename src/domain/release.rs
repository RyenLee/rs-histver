use serde::{Deserialize, Serialize};

/// Rust release information.
///
/// Core data model shared by database, fetcher, and app modules.
/// No dependency on any other business module to avoid circular dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustRelease {
    pub version: String,
    pub date: String,
    pub channel: String,
}

impl RustRelease {
    /// Generate redb primary key: "channel:version:date"
    pub fn db_key(&self) -> String {
        format!("{}:{}:{}", self.channel, self.version, self.date)
    }
}
