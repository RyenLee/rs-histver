use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(super) const DEFAULT_DB_FILENAME: &str = "rs-histver.redb";
const DEFAULT_DATA_DIR: &str = "data";
const DEFAULT_TABLE_PREFIX: &str = "rs_histver";
const CHANNELS: &[&str] = &["stable", "beta", "nightly"];

/// Application configuration.
///
/// Holds database and network settings. Create via [`Config::new()`],
/// [`Config::with_db_path()`], [`Config::from_config_file()`],
/// or [`ConfigBuilder`].
///
/// # Priority order
///
/// Programmatic override → config.toml → hardcoded defaults
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

fn home_dir() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn resolve_path_vars(path_str: &str) -> PathBuf {
    let s = path_str.trim();

    if let Some(stripped) = s.strip_prefix("~/") {
        return home_dir().join(stripped);
    }

    let resolved = s
        .replace("$EXE_DIR", &exe_dir().to_string_lossy())
        .replace("$HOME", &home_dir().to_string_lossy());

    PathBuf::from(resolved)
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

    /// Load configuration from a host project's `config.toml`.
    ///
    /// Supports `[paths].data_dir` with variable substitution (`$EXE_DIR`, `$HOME`, `~/`),
    /// `[rs-histver.database]` (table_prefix, db_file), and `[rs-histver.network]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_config_file(path: impl Into<PathBuf>) -> Result<Self> {
        ConfigBuilder::new().config_file(path).build()
    }

    /// Create a [`ConfigBuilder`] for fine-grained configuration.
    ///
    /// Supports method chaining with config file + programmatic overrides.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

// ---- Config file deserialization ----

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    paths: Option<PathsConfig>,
    #[serde(rename = "rs-histver")]
    rs_histver: Option<RsHistverConfig>,
}

#[derive(Debug, Deserialize)]
struct PathsConfig {
    data_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RsHistverConfig {
    database: Option<DatabaseConfigFile>,
    network: Option<NetworkConfigFile>,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfigFile {
    table_prefix: Option<String>,
    db_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NetworkConfigFile {
    timeout: Option<u64>,
    max_concurrency: Option<usize>,
}

// ---- ConfigBuilder ----

/// Builder for constructing [`Config`] with flexible priority.
///
/// Supports three levels of configuration, from highest to lowest priority:
///
/// 1. **Programmatic override** — `db_path()`, `data_dir()`, `db_file()`, etc.
/// 2. **Config file** — values from `config.toml`, loaded via `config_file()`.
/// 3. **Hardcoded defaults** — applied when no other source provides a value.
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
///
/// // Config file + programmatic overrides
/// let config = ConfigBuilder::new()
///     .config_file("config.toml")
///     .timeout(60)
///     .build()?;
/// ```
pub struct ConfigBuilder {
    config_file_path: Option<PathBuf>,
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
            config_file_path: None,
            db_path_override: None,
            data_dir_override: None,
            db_file_override: None,
            table_prefix_override: None,
            timeout_override: None,
            max_concurrency_override: None,
        }
    }

    /// Load configuration from a host project's `config.toml`.
    ///
    /// File values are lower priority than per-field overrides and higher
    /// priority than hardcoded defaults.
    pub fn config_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_file_path = Some(path.into());
        self
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

    /// Build the final [`Config`], applying priority: programmatic > file > defaults.
    ///
    /// In shared mode (when `db_file()` is set) with the default `table_prefix`,
    /// a warning is printed to stderr to prevent table name collisions.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn build(self) -> Result<Config> {
        let file_cfg = self.load_config_file()?;

        let db_file = self.db_file_override.or_else(|| {
            file_cfg
                .as_ref()
                .and_then(|c| c.rs_histver.as_ref())
                .and_then(|h| h.database.as_ref())
                .and_then(|d| d.db_file.clone())
        });

        let data_dir = self.data_dir_override.or_else(|| {
            file_cfg
                .as_ref()
                .and_then(|c| c.paths.as_ref())
                .and_then(|p| p.data_dir.as_ref())
                .map(|dir| resolve_path_vars(dir))
        });

        let (db_path, shared) = if let Some(path) = self.db_path_override {
            (path, false)
        } else if let Some(file_name) = db_file {
            let dir = data_dir.unwrap_or_else(|| exe_dir().join(DEFAULT_DATA_DIR));
            (dir.join(&file_name), true)
        } else {
            let dir = data_dir.unwrap_or_else(|| exe_dir().join(DEFAULT_DATA_DIR));
            (dir.join(DEFAULT_DB_FILENAME), false)
        };

        let table_prefix = self
            .table_prefix_override
            .or_else(|| {
                file_cfg
                    .as_ref()
                    .and_then(|c| c.rs_histver.as_ref())
                    .and_then(|h| h.database.as_ref())
                    .and_then(|d| d.table_prefix.clone())
            })
            .unwrap_or_else(|| DEFAULT_TABLE_PREFIX.to_string());

        if shared && table_prefix == DEFAULT_TABLE_PREFIX {
            eprintln!(
                "Warning: Shared database mode with default table_prefix \
                 ('{DEFAULT_TABLE_PREFIX}') may cause table name conflicts. \
                 Consider setting [rs-histver.database].table_prefix in config.toml."
            );
        }

        let timeout = self
            .timeout_override
            .or_else(|| {
                file_cfg
                    .as_ref()
                    .and_then(|c| c.rs_histver.as_ref())
                    .and_then(|h| h.network.as_ref())
                    .and_then(|n| n.timeout)
            })
            .unwrap_or(15);

        let max_concurrency = self
            .max_concurrency_override
            .or_else(|| {
                file_cfg
                    .as_ref()
                    .and_then(|c| c.rs_histver.as_ref())
                    .and_then(|h| h.network.as_ref())
                    .and_then(|n| n.max_concurrency)
            })
            .unwrap_or(10);

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

    fn load_config_file(&self) -> Result<Option<ConfigFile>> {
        let path = match &self.config_file_path {
            Some(p) => p,
            None => return Ok(None),
        };

        if !path.exists() {
            return Err(anyhow::anyhow!("Config file not found: {}", path.display()));
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let cfg: ConfigFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(Some(cfg))
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
    fn test_builder_missing_config_file_falls_back() {
        let result = ConfigBuilder::new()
            .config_file("/nonexistent/config.toml")
            .build();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Config file not found"));
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

    #[test]
    fn test_config_file_parse() {
        let toml_str = r#"
[paths]
data_dir = "/opt/myapp/data"

[rs-histver.database]
table_prefix = "myapp_histver"
db_file = "myapp.redb"

[rs-histver.network]
timeout = 30
max_concurrency = 5
"#;
        let cfg: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.paths.unwrap().data_dir.unwrap(), "/opt/myapp/data");
        let rh = cfg.rs_histver.unwrap();
        let db_cfg = rh.database.unwrap();
        assert_eq!(db_cfg.table_prefix.unwrap(), "myapp_histver");
        assert_eq!(db_cfg.db_file.unwrap(), "myapp.redb");
        let net = rh.network.unwrap();
        assert_eq!(net.timeout.unwrap(), 30);
        assert_eq!(net.max_concurrency.unwrap(), 5);
    }

    #[test]
    fn test_config_file_partial() {
        let toml_str = r#"
[paths]
data_dir = "/opt/myapp/data"
"#;
        let cfg: ConfigFile = toml::from_str(toml_str).unwrap();
        assert!(cfg.rs_histver.is_none());
        assert_eq!(cfg.paths.unwrap().data_dir.unwrap(), "/opt/myapp/data");
    }

    #[test]
    fn test_resolve_path_vars_tilde() {
        let resolved = resolve_path_vars("~/myapp/data");
        let home = home_dir();
        assert_eq!(resolved, home.join("myapp/data"));
    }

    #[test]
    fn test_resolve_path_vars_exe_dir() {
        let resolved = resolve_path_vars("$EXE_DIR/data");
        let expected = exe_dir().join("data");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_resolve_path_vars_home_var() {
        let resolved = resolve_path_vars("$HOME/myapp/data");
        let expected = home_dir().join("myapp/data");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_resolve_path_vars_literal() {
        let resolved = resolve_path_vars("/opt/myapp/data");
        assert_eq!(resolved, PathBuf::from("/opt/myapp/data"));
    }

    #[test]
    fn test_resolve_path_vars_mixed() {
        let resolved = resolve_path_vars("$EXE_DIR/$HOME/data");
        assert!(resolved.to_string_lossy().contains("data"));
        assert!(!resolved.to_string_lossy().contains("$EXE_DIR"));
        assert!(!resolved.to_string_lossy().contains("$HOME"));
    }

    #[test]
    fn test_config_file_with_vars() {
        let toml_str = r#"
[paths]
data_dir = "$EXE_DIR/data"
"#;
        let cfg: ConfigFile = toml::from_str(toml_str).unwrap();
        let data_dir_str = cfg.paths.unwrap().data_dir.unwrap();
        let resolved = resolve_path_vars(&data_dir_str);
        assert_eq!(resolved, exe_dir().join("data"));
    }
}
