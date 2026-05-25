use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// 本库默认的数据库文件名
const DEFAULT_DB_FILENAME: &str = "rs-histver.redb";

/// Top-level application configuration.
///
/// Deserialized from `config.toml`, containing database and network settings.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Database connection settings
    pub database: DatabaseConfig,
    /// Network and HTTP client settings
    pub network: NetworkConfig,
}

/// Database configuration section.
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// Path to the redb database file. Empty string uses the OS-specific default path.
    #[serde(default)]
    pub path: String,
    /// Name of the database table for storing release records.
    #[serde(default = "default_table_name")]
    pub table_name: String,
}

/// Network and HTTP client configuration section.
#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    /// HTTP request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Maximum number of concurrent HTTP requests (for beta/nightly probing).
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: usize,
    /// User-Agent header sent with HTTP requests.
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

fn default_table_name() -> String {
    "rust_releases".to_string()
}

fn default_timeout() -> u64 {
    15
}

fn default_max_concurrency() -> usize {
    10
}

fn default_user_agent() -> String {
    "rs-histver/0.1".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: String::new(),
                table_name: default_table_name(),
            },
            network: NetworkConfig {
                timeout: default_timeout(),
                max_concurrency: default_max_concurrency(),
                user_agent: default_user_agent(),
            },
        }
    }
}

impl Config {
    /// Load config from file; returns default if file does not exist
    pub fn load(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Resolve config file path.
    ///
    /// Search order: CLI specified > current directory > executable directory
    pub fn resolve_path(cli_path: Option<&str>) -> PathBuf {
        if let Some(p) = cli_path {
            return PathBuf::from(p);
        }

        let cwd = PathBuf::from("config.toml");
        if cwd.exists() {
            return cwd;
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let p = dir.join("config.toml");
                if p.exists() {
                    return p;
                }
            }
        }

        PathBuf::from("config.toml")
    }

    /// Get the full database file path
    pub fn db_path(&self) -> Result<PathBuf> {
        if !self.database.path.is_empty() {
            let p = PathBuf::from(&self.database.path);
            if p.is_absolute() {
                return Ok(p);
            }
            let exe_dir = std::env::current_exe()
                .context("Failed to get executable path")?
                .parent()
                .context("Executable path has no parent directory")?
                .to_path_buf();
            return Ok(exe_dir.join(p));
        }

        let dir = dirs_data_dir()?;
        Ok(dir.join("rs-histver").join(DEFAULT_DB_FILENAME))
    }

    /// 从目标项目数据库路径推导 rs-histver 的配置（同目录放置策略）。
    ///
    /// 扫描逻辑（按优先级）：
    /// 1. 如果提供了 `target_config_path`，则读取该 TOML 文件，尝试提取目标项目
    ///    的数据库路径，将 `rs-histver.redb` 放在同目录下
    /// 2. 如果提供了 `target_db_path`，则直接以其所在目录作为 rs-histver 数据库目录
    /// 3. 如果提供了 `fallback_dir`，则使用该目录
    /// 4. 全部未提供或扫描失败，则回退到系统默认路径 (`Self::default()`)
    ///
    /// ## 使用示例
    ///
    /// ```no_run
    /// use rs_histver::infra::Config;
    ///
    /// // 扫描目标项目的 config.toml，自动找到其数据库目录
    /// let config = Config::co_locate(
    ///     Some("project-root/app.toml"),   // 目标项目配置文件
    ///     None,                            // 或直接指定目标数据库路径
    ///     Some("project-root/data"),       // 扫描失败时的兜底目录
    /// );
    /// ```
    #[allow(dead_code)]  // pub API 供外部库消费者使用，bin 无需调用
    pub fn co_locate(
        target_config_path: Option<&str>,
        target_db_path: Option<&str>,
        fallback_dir: Option<&str>,
    ) -> Self {
        // 1. 尝试从目标项目配置文件找到数据库路径
        if let Some(cfg_path) = target_config_path {
            if let Some(dir) = scan_target_config_for_db_dir(cfg_path) {
                return Self::with_db_path(dir.join(DEFAULT_DB_FILENAME));
            }
        }

        // 2. 直接从指定的目标数据库路径推导目录
        if let Some(db_path) = target_db_path {
            let p = Path::new(db_path);
            let dir = if p.is_dir() { p.to_path_buf() } else { p.parent().map(|d| d.to_path_buf()).unwrap_or_else(|| PathBuf::from(".")) };
            return Self::with_db_path(dir.join(DEFAULT_DB_FILENAME));
        }

        // 3. 使用兜底目录
        if let Some(dir) = fallback_dir {
            return Self::with_db_path(PathBuf::from(dir).join(DEFAULT_DB_FILENAME));
        }

        // 4. 全部失败，使用系统默认配置
        Self::default()
    }

    /// 构造一个指定数据库文件路径的 Config
    #[allow(dead_code)]
    fn with_db_path(db_path: impl Into<PathBuf>) -> Self {
        Self {
            database: DatabaseConfig {
                path: db_path.into().to_string_lossy().to_string(),
                table_name: default_table_name(),
            },
            network: NetworkConfig {
                timeout: default_timeout(),
                max_concurrency: default_max_concurrency(),
                user_agent: default_user_agent(),
            },
        }
    }
}

/// 扫描目标项目的 TOML 配置文件，尝试提取数据库文件所在目录。
///
/// 支持的常见配置格式：
/// - `[database] path = "data/project.db"`  — 与 rs-histver 相同的段名
/// - `database_url = "sqlite://./data/project.db"`  — SQLite 连接 URL
/// - `database_url = "sqlite:./data/project.db"`  — 无 `//` 前缀的 SQLite URL
/// - `db_path = "./data/project.db"`  — 根级键
///
/// 返回数据库文件所在的目录; 如果解析不到, 返回 `None`
#[allow(dead_code)]
fn scan_target_config_for_db_dir(cfg_path: &str) -> Option<PathBuf> {
    let content = std::fs::read_to_string(cfg_path).ok()?;
    scan_toml_for_db_dir(&content)
}

/// 从 TOML 内容中提取数据库文件所在目录
#[allow(dead_code)]
fn scan_toml_for_db_dir(content: &str) -> Option<PathBuf> {
    let value: toml::Value = content.parse().ok()?;

    // [database] path = "..."
    if let Some(db_path) = value
        .get("database")
        .and_then(|db| db.get("path"))
        .and_then(|v| v.as_str())
    {
        if !db_path.is_empty() {
            return dir_of(db_path);
        }
    }

    // database_url = "sqlite://..."
    if let Some(url) = value.get("database_url").and_then(|v| v.as_str()) {
        if let Some(db_path) = extract_file_path_from_db_url(url) {
            return dir_of(&db_path);
        }
    }

    // db_path = "..."
    if let Some(db_path) = value.get("db_path").and_then(|v| v.as_str()) {
        if !db_path.is_empty() {
            return dir_of(db_path);
        }
    }

    None
}

/// 从数据库 URL 提取文件系统路径 (支持 SQLite 等文件型数据库)
#[allow(dead_code)]
fn extract_file_path_from_db_url(url: &str) -> Option<String> {
    // sqlite://./data/project.db → ./data/project.db
    if let Some(path) = url.strip_prefix("sqlite://") {
        if !path.is_empty() {
            return Some(path.to_string());
        }
    }
    // sqlite:./data/project.db → ./data/project.db
    if let Some(path) = url.strip_prefix("sqlite:") {
        if !path.is_empty() {
            return Some(path.to_string());
        }
    }
    None
}

/// 从数据库文件路径提取其所在目录
#[allow(dead_code)]
fn dir_of(db_path: &str) -> Option<PathBuf> {
    let p = Path::new(db_path);
    // 如果路径存在且是目录，直接返回
    if p.exists() && p.is_dir() {
        return Some(p.to_path_buf());
    }
    // 提取父目录；如果没有父目录（如单文件名或 `data`），使用当前目录
    match p.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => Some(parent.to_path_buf()),
        _ => Some(PathBuf::from(".")),
    }
}

fn dirs_data_dir() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .context("LOCALAPPDATA environment variable not set")
    } else {
        dirs_home_dir().map(|h| h.join(".local").join("share"))
    }
}

fn dirs_home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .context("Failed to determine home directory")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_toml_database_path() {
        let dir = scan_toml_for_db_dir(r#"[database]
path = "data/project.db"
"#);
        assert_eq!(dir, Some(PathBuf::from("data")));
    }

    #[test]
    fn test_scan_toml_database_url() {
        let dir = scan_toml_for_db_dir(r#"database_url = "sqlite://./data/project.db"
"#);
        assert_eq!(dir, Some(PathBuf::from("./data")));
    }

    #[test]
    fn test_scan_toml_db_path_key() {
        let dir = scan_toml_for_db_dir(r#"db_path = "./data/project.db"
"#);
        assert_eq!(dir, Some(PathBuf::from("./data")));
    }

    #[test]
    fn test_scan_toml_empty_path() {
        let dir = scan_toml_for_db_dir(r#"[database]
path = ""
"#);
        assert!(dir.is_none());
    }

    #[test]
    fn test_scan_toml_no_match() {
        let dir = scan_toml_for_db_dir(r#"[app]
name = "hello"
"#);
        assert!(dir.is_none());
    }

    #[test]
    fn test_scan_toml_database_url_sqlite() {
        let url = extract_file_path_from_db_url("sqlite://./data/project.db");
        assert_eq!(url, Some("./data/project.db".to_string()));
    }

    #[test]
    fn test_scan_toml_database_url_sqlite_no_slash() {
        let url = extract_file_path_from_db_url("sqlite:./data/project.db");
        assert_eq!(url, Some("./data/project.db".to_string()));
    }

    #[test]
    fn test_dir_of_file() {
        let dir = dir_of("data/project.db").unwrap();
        assert_eq!(dir, PathBuf::from("data"));
    }

    #[test]
    fn test_dir_of_directory() {
        // "data" 目录不存在时，parent() 返回空路径，fallback 到 "."
        let dir = dir_of("data").unwrap();
        assert_eq!(dir, PathBuf::from("."));
    }

    #[test]
    fn test_co_locate_with_target_db_path() {
        let config = Config::co_locate(None, Some("data/project.db"), None);
        let expected = PathBuf::from("data/rs-histver.redb");
        let got = std::path::Path::new(&config.database.path);
        // 规范化比较
        let expected = expected.iter().collect::<Vec<_>>();
        let got = got.iter().collect::<Vec<_>>();
        assert_eq!(got, expected);
    }

    #[test]
    fn test_co_locate_with_fallback_dir() {
        let config = Config::co_locate(None, None, Some("data"));
        let expected = PathBuf::from("data/rs-histver.redb");
        let got = std::path::Path::new(&config.database.path);
        let expected = expected.iter().collect::<Vec<_>>();
        let got = got.iter().collect::<Vec<_>>();
        assert_eq!(got, expected);
    }

    #[test]
    fn test_co_locate_with_none_defaults() {
        let config = Config::co_locate(None, None, None);
        // 全 None → 回退到 Self::default()，path 为空字符串（由 db_path() 在运行时解析到系统默认路径）
        assert!(config.database.path.is_empty());
        assert_eq!(config.database.table_name, "rust_releases");
        assert_eq!(config.network.timeout, 15);
    }
}
