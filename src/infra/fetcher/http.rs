use anyhow::{Context, Result};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Semaphore;

use crate::domain::RustRelease;
use crate::infra::Config;

const DATE_CHANNEL_TOML: &str = "https://static.rust-lang.org/dist";

/// Cached regex for extracting date from version string.
static DATE_RE: OnceLock<regex_lite::Regex> = OnceLock::new();

fn date_regex() -> &'static regex_lite::Regex {
    DATE_RE.get_or_init(|| regex_lite::Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap())
}

/// Build an HTTP client
pub(super) fn build_client(config: &Config) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(&config.network.user_agent)
        .timeout(std::time::Duration::from_secs(config.network.timeout))
        .build()
        .context("Failed to build HTTP client")
}

/// Build an HTTP client with fallback to a default one on error
pub(super) fn build_client_fallback(config: &Config) -> reqwest::Client {
    build_client(config).unwrap_or_else(|_| {
        reqwest::Client::builder()
            .user_agent(&config.network.user_agent)
            .build()
            .unwrap()
    })
}

/// Concurrently probe channel TOML for the recent N days (including today)
pub(super) async fn probe_channel_history(
    config: &Config,
    channel: &str,
    days: u32,
) -> Vec<RustRelease> {
    let client = Arc::new(build_client_fallback(config));
    let semaphore = Arc::new(Semaphore::new(config.network.max_concurrency));
    let today = chrono::Local::now().date_naive();

    let mut handles = Vec::new();

    for i in 0..days {
        let date = today - chrono::Duration::days(i64::from(i));
        let date_str = date.format("%Y-%m-%d").to_string();
        let client = client.clone();
        let sem = semaphore.clone();
        let channel = channel.to_string();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let url = format!(
                "{DATE_CHANNEL_TOML}/{date_str}/channel-rust-{channel}.toml"
            );
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => match resp.text().await {
                    Ok(text) => parse_channel_toml_version(&text, &channel, &date_str),
                    Err(_) => None,
                },
                _ => None,
            }
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(Some(r)) = handle.await {
            results.push(r);
        }
    }

    results.sort_by(|a, b| b.date.cmp(&a.date));
    results
}

/// Parse the version field from [pkg.rust] section in a channel TOML
fn parse_channel_toml_version(
    text: &str,
    channel: &str,
    fallback_date: &str,
) -> Option<RustRelease> {
    let marker = "[pkg.rust]";
    let pos = text.find(marker)?;
    let after = &text[pos + marker.len()..];

    let version_prefix = r#"version = ""#;
    let vpos = after.find(version_prefix)?;
    let rest = &after[vpos + version_prefix.len()..];
    let end = rest.find('"')?;
    let version_str = &rest[..end];

    let date = date_regex()
        .captures(version_str).map_or_else(|| fallback_date.to_string(), |c| c[1].to_string());

    Some(RustRelease {
        version: version_str.to_string(),
        date,
        channel: channel.to_string(),
    })
}
