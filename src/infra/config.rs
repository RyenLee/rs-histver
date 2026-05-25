use std::path::{Path, PathBuf};

/// Default database file name
const DEFAULT_DB_FILENAME: &str = "rs-histver.redb";

/// Default database subdirectory name
const DEFAULT_DATA_DIR: &str = "data";

/// Default database table name prefix (actual tables: {prefix}_stable, {prefix}_beta, {prefix}_nightly)
const DEFAULT_TABLE_PREFIX: &str = "rs_histver";

/// Supported release channels
const CHANNELS: &[&str] = &["stable", "beta", "nightly"];

/// Application configuration.
///
/// Contains database and network settings with sensible defaults.
/// No longer reads from config files; all values are hardcoded or
/// set programmatically.
#[derive(Debug, Clone)]
pub struct Config {
    /// Database settings
    pub database: DatabaseConfig,
    /// Network and HTTP client settings
    pub network: NetworkConfig,
}

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the redb database file.
    /// Defaults to `<exe_dir>/data/rs-histver.redb`.
    pub path: PathBuf,
    /// Prefix for database table names.
    /// Actual tables: `{prefix}_stable`, `{prefix}_beta`, `{prefix}_nightly`.
    pub table_prefix: String,
}

impl DatabaseConfig {
    /// Get the full table name for a given channel.
    pub fn table_name_for(&self, channel: &str) -> String {
        format!("{}_{}", self.table_prefix, channel)
    }

    /// Get all channel table names.
    pub fn all_table_names(&self) -> Vec<String> {
        CHANNELS.iter().map(|ch| self.table_name_for(ch)).collect()
    }
}

/// Network and HTTP client configuration.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// HTTP request timeout in seconds.
    pub timeout: u64,
    /// Maximum number of concurrent HTTP requests (for beta/nightly probing).
    pub max_concurrency: usize,
    /// User-Agent header sent with HTTP requests.
    pub user_agent: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: default_db_path(),
                table_prefix: DEFAULT_TABLE_PREFIX.to_string(),
            },
            network: NetworkConfig {
                timeout: 15,
                max_concurrency: 10,
                user_agent: format!("rs-histver/{}", env!("CARGO_PKG_VERSION")),
            },
        }
    }
}

/// Compute the default database path: `<exe_dir>/data/rs-histver.redb`
fn default_db_path() -> PathBuf {
    exe_dir().join(DEFAULT_DATA_DIR).join(DEFAULT_DB_FILENAME)
}

/// Get the directory containing the current executable.
fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Config {
    /// Create a new Config with the default database path (`<exe_dir>/data/rs-histver.redb`).
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a Config with a custom database path.
    pub fn with_db_path(db_path: impl Into<PathBuf>) -> Self {
        Self {
            database: DatabaseConfig {
                path: db_path.into(),
                table_prefix: DEFAULT_TABLE_PREFIX.to_string(),
            },
            network: NetworkConfig {
                timeout: 15,
                max_concurrency: 10,
                user_agent: format!("rs-histver/{}", env!("CARGO_PKG_VERSION")),
            },
        }
    }

    /// Get the database file path as a reference.
    #[must_use] 
    pub fn db_path(&self) -> &Path {
        &self.database.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database.table_prefix, "rs_histver");
        assert_eq!(config.network.timeout, 15);
        assert_eq!(config.network.max_concurrency, 10);
        assert!(config.database.path.to_string_lossy().contains("data"));
        assert!(config
            .database
            .path
            .to_string_lossy()
            .contains("rs-histver.redb"));
    }

    #[test]
    fn test_with_db_path() {
        let config = Config::with_db_path("/tmp/custom.redb");
        assert_eq!(config.database.path, PathBuf::from("/tmp/custom.redb"));
        assert_eq!(config.database.table_prefix, "rs_histver");
    }

    #[test]
    fn test_db_path_returns_configured_path() {
        let config = Config::with_db_path("/tmp/test.redb");
        let path = config.db_path();
        assert_eq!(path, Path::new("/tmp/test.redb"));
    }

    #[test]
    fn test_table_name_for() {
        let config = Config::default();
        assert_eq!(
            config.database.table_name_for("stable"),
            "rs_histver_stable"
        );
        assert_eq!(config.database.table_name_for("beta"), "rs_histver_beta");
        assert_eq!(
            config.database.table_name_for("nightly"),
            "rs_histver_nightly"
        );
    }

    #[test]
    fn test_all_table_names() {
        let config = Config::default();
        let names = config.database.all_table_names();
        assert_eq!(
            names,
            vec!["rs_histver_stable", "rs_histver_beta", "rs_histver_nightly"]
        );
    }
}
