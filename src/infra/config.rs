use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) const DEFAULT_DB_FILENAME: &str = "rs-histver.redb";
const DEFAULT_DATA_DIR: &str = "data";
const DEFAULT_TABLE_PREFIX: &str = "rs_histver";
const CHANNELS: &[&str] = &["stable", "beta", "nightly"];

/// Application configuration.
///
/// Holds database and network settings. Create via [`Config::new()`],
/// [`Config::with_db_path()`], or [`ConfigBuilder`].
///
/// # Priority order
///
/// Programmatic override → hardcoded defaults
#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub network: NetworkConfig,
}

/// Database configuration.
///
/// Determines the database file path, table naming strategy, and whether
/// the database is shared with a host project.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub table_prefix: String,
    pub shared: bool,
}

impl DatabaseConfig {
    /// Build the full table name for a given channel.
    ///
    /// Format: `{table_prefix}_{channel}` (e.g. `rs_histver_stable`).
    #[must_use]
    pub fn table_name_for(&self, channel: &str) -> String {
        format!("{}_{}", self.table_prefix, channel)
    }

    /// Return table names for all three channels (stable, beta, nightly).
    #[must_use]
    pub fn all_table_names(&self) -> Vec<String> {
        CHANNELS.iter().map(|ch| self.table_name_for(ch)).collect()
    }
}

/// Network configuration for HTTP requests.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Request timeout in seconds.
    pub timeout: u64,
    /// Maximum concurrent requests.
    pub max_concurrency: usize,
    /// User-Agent header sent with each request.
    pub user_agent: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: default_db_path(),
                table_prefix: DEFAULT_TABLE_PREFIX.to_string(),
                shared: false,
            },
            network: NetworkConfig {
                timeout: 15,
                max_concurrency: 10,
                user_agent: format!("rs-histver/{}", env!("CARGO_PKG_VERSION")),
            },
        }
    }
}

fn default_db_path() -> PathBuf {
    exe_dir().join(DEFAULT_DATA_DIR).join(DEFAULT_DB_FILENAME)
}

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Config {
    /// Create a new `Config` with default values.
    ///
    /// Database path defaults to `<exe_dir>/data/rs-histver.redb`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `Config` with a custom database file path.
    ///
    /// Useful when the database should be stored at a specific location.
    pub fn with_db_path(db_path: impl Into<PathBuf>) -> Self {
        Self {
            database: DatabaseConfig {
                path: db_path.into(),
                table_prefix: DEFAULT_TABLE_PREFIX.to_string(),
                shared: false,
            },
            network: NetworkConfig {
                timeout: 15,
                max_concurrency: 10,
                user_agent: format!("rs-histver/{}", env!("CARGO_PKG_VERSION")),
            },
        }
    }

    /// Return the configured database file path.
    #[must_use]
    pub fn db_path(&self) -> &Path {
        &self.database.path
    }

    /// Create a [`ConfigBuilder`] for fine-grained configuration.
    ///
    /// Supports method chaining with programmatic overrides.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

// ---- ConfigBuilder ----

/// Builder for constructing [`Config`] with flexible overrides.
///
/// Supports two levels of configuration, from highest to lowest priority:
///
/// 1. **Programmatic override** — `db_path()`, `data_dir()`, `db_file()`, etc.
/// 2. **Hardcoded defaults** — applied when no override provides a value.
///
/// # Examples
///
/// ```ignore
/// use rs_histver::ConfigBuilder;
///
/// // Standalone mode (default)
/// let config = ConfigBuilder::new()
///     .data_dir("/app/data")
///     .build()?;
///
/// // Shared mode (host also uses redb)
/// let config = ConfigBuilder::new()
///     .data_dir("/app/data")
///     .db_file("myapp.redb")
///     .table_prefix("myapp_histver")
///     .build()?;
/// ```
pub struct ConfigBuilder {
    db_path_override: Option<PathBuf>,
    data_dir_override: Option<PathBuf>,
    db_file_override: Option<String>,
    table_prefix_override: Option<String>,
    timeout_override: Option<u64>,
    max_concurrency_override: Option<usize>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    /// Create a new `ConfigBuilder` with no overrides set.
    pub fn new() -> Self {
        Self {
            db_path_override: None,
            data_dir_override: None,
            db_file_override: None,
            table_prefix_override: None,
            timeout_override: None,
            max_concurrency_override: None,
        }
    }

    /// Set a complete database file path (highest priority, standalone mode).
    ///
    /// When set, `data_dir()` and `db_file()` are ignored.
    pub fn db_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.db_path_override = Some(path.into());
        self
    }

    /// Set the data directory for database files.
    ///
    /// Without `db_file()`, creates `<data_dir>/rs-histver.redb` (standalone mode).
    /// With `db_file()`, opens `<data_dir>/<db_file>` (shared mode).
    pub fn data_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.data_dir_override = Some(dir.into());
        self
    }

    /// Set the database filename for shared mode.
    ///
    /// When set, the system opens the host's redb database file instead of
    /// creating a standalone `rs-histver.redb`. Combine with `table_prefix()`
    /// to avoid name collisions.
    pub fn db_file(mut self, file: impl Into<String>) -> Self {
        self.db_file_override = Some(file.into());
        self
    }

    /// Set the prefix for database table names.
    ///
    /// Table names follow the pattern `{prefix}_{channel}`.
    /// Default: `"rs_histver"`.
    pub fn table_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.table_prefix_override = Some(prefix.into());
        self
    }

    /// Set the HTTP request timeout in seconds. Default: `15`.
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout_override = Some(timeout);
        self
    }

    /// Set the maximum number of concurrent HTTP requests. Default: `10`.
    pub fn max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency_override = Some(max);
        self
    }

    /// Build the final [`Config`], applying priority: programmatic > defaults.
    ///
    /// In shared mode (when `db_file()` is set) with the default `table_prefix`,
    /// a warning is printed to stderr to prevent table name collisions.
    pub fn build(self) -> Result<Config> {
        let (db_path, shared) = if let Some(path) = self.db_path_override {
            (path, false)
        } else if let Some(file_name) = self.db_file_override {
            let dir = self
                .data_dir_override
                .unwrap_or_else(|| exe_dir().join(DEFAULT_DATA_DIR));
            (dir.join(&file_name), true)
        } else {
            let dir = self
                .data_dir_override
                .unwrap_or_else(|| exe_dir().join(DEFAULT_DATA_DIR));
            (dir.join(DEFAULT_DB_FILENAME), false)
        };

        let table_prefix = self
            .table_prefix_override
            .unwrap_or_else(|| DEFAULT_TABLE_PREFIX.to_string());

        if shared && table_prefix == DEFAULT_TABLE_PREFIX {
            eprintln!(
                "Warning: Shared database mode with default table_prefix \
                 ('{DEFAULT_TABLE_PREFIX}') may cause table name conflicts. \
                 Consider setting a custom table_prefix."
            );
        }

        let timeout = self.timeout_override.unwrap_or(15);
        let max_concurrency = self.max_concurrency_override.unwrap_or(10);

        Ok(Config {
            database: DatabaseConfig {
                path: db_path,
                table_prefix,
                shared,
            },
            network: NetworkConfig {
                timeout,
                max_concurrency,
                user_agent: format!("rs-histver/{}", env!("CARGO_PKG_VERSION")),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database.table_prefix, "rs_histver");
        assert!(!config.database.shared);
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
        assert!(!config.database.shared);
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

    #[test]
    fn test_builder_db_path_overrides_data_dir() {
        let config = ConfigBuilder::new()
            .data_dir("/tmp/mydata")
            .db_path("/tmp/custom.redb")
            .build()
            .unwrap();
        assert_eq!(config.database.path, PathBuf::from("/tmp/custom.redb"));
        assert!(!config.database.shared);
    }

    #[test]
    fn test_builder_data_dir_appends_filename() {
        let config = ConfigBuilder::new()
            .data_dir("/tmp/mydata")
            .build()
            .unwrap();
        assert_eq!(
            config.database.path,
            PathBuf::from("/tmp/mydata/rs-histver.redb")
        );
        assert!(!config.database.shared);
    }

    #[test]
    fn test_builder_default_fallback() {
        let config = ConfigBuilder::new().build().unwrap();
        assert_eq!(config.database.table_prefix, "rs_histver");
        assert!(!config.database.shared);
        assert_eq!(config.network.timeout, 15);
        assert_eq!(config.network.max_concurrency, 10);
    }

    #[test]
    fn test_builder_programmatic_overrides() {
        let config = ConfigBuilder::new()
            .table_prefix("myapp")
            .timeout(30)
            .max_concurrency(5)
            .build()
            .unwrap();
        assert_eq!(config.database.table_prefix, "myapp");
        assert!(!config.database.shared);
        assert_eq!(config.network.timeout, 30);
        assert_eq!(config.network.max_concurrency, 5);
    }

    #[test]
    fn test_builder_db_file_shared_mode() {
        let config = ConfigBuilder::new()
            .data_dir("/tmp/mydata")
            .db_file("myapp.redb")
            .table_prefix("myapp_histver")
            .build()
            .unwrap();
        assert_eq!(
            config.database.path,
            PathBuf::from("/tmp/mydata/myapp.redb")
        );
        assert_eq!(config.database.table_prefix, "myapp_histver");
        assert!(config.database.shared);
    }

    #[test]
    fn test_builder_db_path_overrides_db_file() {
        let config = ConfigBuilder::new()
            .data_dir("/tmp/mydata")
            .db_file("myapp.redb")
            .db_path("/tmp/custom.redb")
            .build()
            .unwrap();
        assert_eq!(config.database.path, PathBuf::from("/tmp/custom.redb"));
        assert!(!config.database.shared);
    }

    #[test]
    fn test_builder_data_dir_without_db_file_standalone() {
        let config = ConfigBuilder::new()
            .data_dir("/tmp/mydata")
            .build()
            .unwrap();
        assert_eq!(
            config.database.path,
            PathBuf::from("/tmp/mydata/rs-histver.redb")
        );
        assert!(!config.database.shared);
    }
}
