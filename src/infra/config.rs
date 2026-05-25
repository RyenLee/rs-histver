use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub network: NetworkConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub path: String,
    #[serde(default = "default_table_name")]
    pub table_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: usize,
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
        Ok(dir.join("rs-histver").join("releases.redb"))
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
