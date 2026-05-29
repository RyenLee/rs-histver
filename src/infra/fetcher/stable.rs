use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::OnceLock;

use crate::domain::RustRelease;
use crate::infra::Config;

use super::http::build_client;
use super::ReleaseFetcher;

const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/rust-lang/rust/releases";
const RELEASES_MD_URL: &str = "https://raw.githubusercontent.com/rust-lang/rust/master/RELEASES.md";

/// Cached regex for parsing RELEASES.md version lines.
static RELEASES_MD_RE: OnceLock<regex_lite::Regex> = OnceLock::new();

fn releases_md_regex() -> &'static regex_lite::Regex {
    RELEASES_MD_RE.get_or_init(|| {
        regex_lite::Regex::new(r"Version\s+(\d+\.\d+\.\d+)\s+\((\d{4}-\d{2}-\d{2})\)")
            .expect("RELEASES_MD_RE regex pattern is statically valid")
    })
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    published_at: String,
    prerelease: bool,
    draft: bool,
}

/// Stable channel fetcher.
///
/// Uses GitHub Releases API by default, or `RELEASES.md` for full historical
/// data when `full` is enabled.
pub(super) struct StableFetcher {
    full: bool,
}

impl StableFetcher {
    pub fn new(full: bool) -> Self {
        Self { full }
    }
}

#[async_trait]
impl ReleaseFetcher for StableFetcher {
    fn channel_name(&self) -> &'static str {
        "stable"
    }

    fn source_description(&self) -> &str {
        if self.full {
            "RELEASES.md (full history)"
        } else {
            "GitHub Releases API"
        }
    }

    async fn fetch(&self, config: &Config) -> Result<Vec<RustRelease>> {
        if self.full {
            fetch_from_releases_md(config).await
        } else {
            fetch_from_github(config).await
        }
    }
}

async fn fetch_from_github(config: &Config) -> Result<Vec<RustRelease>> {
    let client = build_client(config)?;
    let mut all = Vec::new();
    let mut page = 1u32;

    loop {
        let url = format!("{GITHUB_RELEASES_API}?per_page=100&page={page}");
        let resp = client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to request GitHub Releases API (page {page})"))?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "GitHub API returned error: {} (page {})",
                resp.status(),
                page
            );
        }

        let releases: Vec<GhRelease> = resp.json().await?;
        if releases.is_empty() {
            break;
        }

        for r in &releases {
            if r.draft || r.prerelease {
                continue;
            }
            let version = r
                .tag_name
                .strip_prefix('v')
                .unwrap_or(&r.tag_name)
                .to_string();
            let date = r
                .published_at
                .get(..10)
                .unwrap_or(&r.published_at)
                .to_string();
            all.push(RustRelease {
                version,
                date,
                channel: "stable".to_string(),
            });
        }

        page += 1;
        if page > 15 {
            break;
        }
    }

    Ok(all)
}

async fn fetch_from_releases_md(config: &Config) -> Result<Vec<RustRelease>> {
    let text = build_client(config)?
        .get(RELEASES_MD_URL)
        .send()
        .await
        .context("Failed to request RELEASES.md")?
        .text()
        .await
        .context("Failed to read RELEASES.md content")?;

    let mut releases = Vec::new();
    for cap in releases_md_regex().captures_iter(&text) {
        releases.push(RustRelease {
            version: cap[1].to_string(),
            date: cap[2].to_string(),
            channel: "stable".to_string(),
        });
    }
    Ok(releases)
}
