# rs-histver

[![Crates.io](https://img.shields.io/crates/v/rs-histver.svg)](https://crates.io/crates/rs-histver)
[![Documentation](https://docs.rs/rs-histver/badge.svg)](https://docs.rs/rs-histver)
[![License](https://img.shields.io/crates/l/rs-histver.svg)](https://github.com/RyenLee/rs-histver#license)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A Rust library and CLI tool for querying historical release versions of the Rust programming language (stable, beta, nightly) via GitHub Releases API and rust-lang.org distribution server.

**As a library**: One function call, pure in-memory result, zero file-system side effects.  
**As a CLI**: Terminal-based query tool with formatted table output.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Library Usage](#library-usage)
- [CLI Usage](#cli-usage)
- [Architecture](#architecture)
- [API Reference](#api-reference)
- [Data Sources](#data-sources)
- [Development](#development)
- [License](#license)

## Features

- **Dual Mode**: Use as a Rust library or standalone CLI tool
- **Zero Side Effects**: Library mode has no database, config file, or file system dependencies
- **Multiple Channels**: Query stable, beta, and nightly release channels
- **Flexible Data Sources**: GitHub API for recent releases, RELEASES.md for complete history
- **Concurrent Fetching**: Configurable concurrency control with semaphore-based rate limiting
- **Type-Safe**: Leverages Rust's type system for compile-time error prevention
- **Well-Documented**: Comprehensive inline documentation with examples

## Installation

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rs-histver = "0.4"
```

### CLI Binary

```bash
cargo install rs-histver
```

Or build from source:

```bash
git clone https://github.com/RyenLee/rs-histver.git
cd rs-histver
cargo build --release
```

## Library Usage

### Basic Example

```rust
use rs_histver::{fetch_releases, FetchOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Fetch stable releases (default: 30-day history via GitHub API)
    let releases = fetch_releases("stable", FetchOptions::default()).await?;
    
    for r in &releases {
        println!("{} ({}) [{}]", r.version, r.date, r.channel);
    }
    
    Ok(())
}
```

### Advanced Usage

```rust
use rs_histver::{fetch_releases, FetchOptions};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Full stable history from RELEASES.md
    let all_stable = fetch_releases(
        "stable",
        FetchOptions::new().full_history(true)
    ).await?;
    
    // Recent 7 days of nightly builds
    let recent_nightly = fetch_releases(
        "nightly",
        FetchOptions::new().probe_days(7)
    ).await?;
    
    // Custom timeout and concurrency
    let releases = fetch_releases(
        "beta",
        FetchOptions::new()
            .timeout(Duration::from_secs(30))
            .max_concurrency(5)
            .user_agent("my-app/1.0")
    ).await?;
    
    Ok(())
}
```

### Low-Level API

For advanced use cases, use the `create_fetcher` factory function and `ReleaseFetcher` trait directly:

```rust
use rs_histver::{create_fetcher, ReleaseFetcher, NetworkConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fetcher = create_fetcher("stable", false, 30)?;
    
    let network = NetworkConfig::new()
        .timeout_secs(30)
        .max_concurrency(5)
        .user_agent("my-app/1.0");
    
    let releases = fetcher.fetch(&network).await?;
    
    println!("Channel: {}", fetcher.channel_name());
    println!("Source: {}", fetcher.source_description());
    
    Ok(())
}
```

## CLI Usage

### Quick Start

```bash
# Fetch stable releases (default: GitHub API, recent 30 days)
rs-histver fetch

# Fetch full stable history from RELEASES.md
rs-histver fetch --full

# Fetch nightly releases from the last 30 days
rs-histver fetch -c nightly

# Fetch beta releases from the last 14 days
rs-histver fetch -c beta --days 14
```

### Commands

| Command | Description |
|---------|-------------|
| `fetch [-c CHANNEL] [--full] [-d DAYS]` | Fetch release data from remote source |

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `-c, --channel <CHANNEL>` | `stable` | Release channel: stable, beta, or nightly |
| `--full` | `false` | Use RELEASES.md for complete stable history |
| `-d, --days <DAYS>` | `30` | Days to probe for beta/nightly channels |

### Examples

```bash
# Stable channel (GitHub API, ~1500 releases)
rs-histver fetch -c stable

# Stable channel (RELEASES.md, complete history)
rs-histver fetch -c stable --full

# Beta channel (probe last 30 days)
rs-histver fetch -c beta

# Nightly channel (probe last 7 days)
rs-histver fetch -c nightly --days 7
```

## Architecture

### Project Structure

```
rs-histver/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # CLI entry point
│   ├── constants.rs        # Centralized constants
│   ├── domain.rs           # Domain models
│   │   └── release.rs      # RustRelease struct
│   ├── options.rs          # FetchOptions & NetworkConfig
│   ├── app/                # Application layer
│   │   ├── handler.rs      # Business logic
│   │   └── display.rs      # Table formatting
│   ├── cli/                # CLI layer
│   │   └── types.rs        # Clap definitions
│   └── infra/               # Infrastructure layer
│       ├── fetcher.rs      # ReleaseFetcher trait & factory
│       └── fetcher/
│           ├── stable.rs   # Stable channel implementation
│           ├── beta.rs     # Beta channel implementation
│           ├── nightly.rs  # Nightly channel implementation
│           └── http.rs     # HTTP client & utilities
└── Cargo.toml
```

### Layered Architecture

```
┌─────────────────────────────────────────┐
│  CLI Layer (src/cli, src/main)          │
│  - Argument parsing (clap)               │
│  - Command routing                       │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  Application Layer (src/app)            │
│  - Business orchestration (handler)     │
│  - Result presentation (display)        │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  Domain Layer (src/domain)              │
│  - Core model (RustRelease)             │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  Infrastructure Layer (src/infra)       │
│  - HTTP client (http)                   │
│  - Data fetching (fetcher)              │
│  - Channel implementations               │
└─────────────────────────────────────────┘
```

### Design Patterns

#### Strategy Pattern

The `ReleaseFetcher` trait defines a common interface for fetching release data, with each channel implementing its own strategy:

```rust
#[async_trait]
pub trait ReleaseFetcher: Send + Sync {
    fn channel_name(&self) -> &str;
    fn source_description(&self) -> &str;
    async fn fetch(&self, network: &NetworkConfig) -> Result<Vec<RustRelease>>;
}
```

#### Factory Method

The `create_fetcher()` function creates the appropriate fetcher based on channel name:

```rust
pub fn create_fetcher(channel: &str, full: bool, days: u32) -> Result<Box<dyn ReleaseFetcher>> {
    match channel {
        "stable" => Ok(Box::new(StableFetcher::new(full))),
        "beta" => Ok(Box::new(BetaFetcher::new(days))),
        "nightly" => Ok(Box::new(NightlyFetcher::new(days))),
        _ => anyhow::bail!("Unknown channel: '{}'", channel),
    }
}
```

#### Builder Pattern

`FetchOptions` and `NetworkConfig` use builder pattern for fluent configuration:

```rust
let opts = FetchOptions::new()
    .full_history(true)
    .probe_days(7)
    .timeout(Duration::from_secs(30))
    .max_concurrency(5);
```

## API Reference

### Core Functions

#### `fetch_releases`

```rust
pub async fn fetch_releases(channel: &str, opts: FetchOptions) -> Result<Vec<RustRelease>>
```

Fetch Rust release data for a given channel.

**Parameters:**
- `channel`: One of `"stable"`, `"beta"`, or `"nightly"`
- `opts`: Query configuration (`FetchOptions`)

**Returns:**
- `Vec<RustRelease>` sorted by date descending

**Errors:**
- Unknown channel name
- Network request failure

### Structs

#### `RustRelease`

```rust
pub struct RustRelease {
    pub version: String,  // e.g., "1.75.0", "1.76.0-nightly"
    pub date: String,     // YYYY-MM-DD format
    pub channel: String,  // "stable", "beta", or "nightly"
}
```

#### `FetchOptions`

```rust
pub struct FetchOptions {
    pub full_history: bool,      // Stable: use RELEASES.md (default: false)
    pub probe_days: u32,         // Beta/Nightly: days to probe (default: 30)
    pub timeout: Duration,       // HTTP timeout (default: 15s)
    pub max_concurrency: usize,  // Max concurrent requests (default: 10)
    pub user_agent: String,      // User-Agent header
}
```

**Methods:**
- `new()` → Create with defaults
- `full_history(bool)` → Enable RELEASES.md mode
- `probe_days(u32)` → Set probe days
- `timeout(Duration)` → Set HTTP timeout
- `max_concurrency(usize)` → Set concurrency limit
- `user_agent(string)` → Set User-Agent

#### `NetworkConfig`

```rust
pub struct NetworkConfig {
    pub timeout: u64,            // Timeout in seconds
    pub max_concurrency: usize,  // Max concurrent requests
    pub user_agent: String,      // User-Agent header
}
```

**Methods:**
- `new()` → Create with defaults
- `timeout_secs(u64)` → Set timeout
- `max_concurrency(usize)` → Set concurrency
- `user_agent(string)` → Set User-Agent

### Traits

#### `ReleaseFetcher`

```rust
#[async_trait]
pub trait ReleaseFetcher: Send + Sync {
    fn channel_name(&self) -> &str;
    fn source_description(&self) -> &str;
    async fn fetch(&self, network: &NetworkConfig) -> Result<Vec<RustRelease>>;
}
```

### Factory Function

#### `create_fetcher`

```rust
pub fn create_fetcher(channel: &str, full: bool, days: u32) -> Result<Box<dyn ReleaseFetcher>>
```

Create a channel-specific fetcher.

**Parameters:**
- `channel`: One of `"stable"`, `"beta"`, or `"nightly"`
- `full`: If `true`, stable fetcher uses RELEASES.md
- `days`: Days to probe (beta/nightly only)

**Returns:**
- `Box<dyn ReleaseFetcher>` for the specified channel

## Data Sources

### Stable Channel

| Mode | Source | Coverage |
|------|--------|----------|
| Default | GitHub Releases API | ~1500 releases (~10 years) |
| `--full` | RELEASES.md | Complete history (all releases) |

**GitHub API:**
- Endpoint: `https://api.github.com/repos/rust-lang/rust/releases`
- Pagination: 100 per page, max 15 pages
- Filters: Excludes drafts and prereleases

**RELEASES.md:**
- URL: `https://raw.githubusercontent.com/rust-lang/rust/master/RELEASES.md`
- Format: Parses `Version X.Y.Z (YYYY-MM-DD)` lines

### Beta Channel

- Source: `https://static.rust-lang.org/dist/{date}/channel-rust-beta.toml`
- Method: Date probing for recent N days
- Version extraction: Parses `[pkg.rust]` section

### Nightly Channel

- Source: `https://static.rust-lang.org/dist/{date}/channel-rust-nightly.toml`
- Method: Date probing for recent N days
- Version extraction: Parses `[pkg.rust]` section

## Constants

All hardcoded values are centralized in `src/constants.rs`:

```rust
// Channel names
pub const CHANNEL_STABLE: &str = "stable";
pub const CHANNEL_BETA: &str = "beta";
pub const CHANNEL_NIGHTLY: &str = "nightly";

// Default configuration
pub const DEFAULT_PROBE_DAYS: u32 = 30;
pub const DEFAULT_TIMEOUT_SECS: u64 = 15;
pub const DEFAULT_MAX_CONCURRENCY: usize = 10;

// GitHub API
pub const GITHUB_PER_PAGE: u32 = 100;
pub const GITHUB_MAX_PAGES: u32 = 15;

// URLs
pub const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/rust-lang/rust/releases";
pub const RELEASES_MD_URL: &str = "https://raw.githubusercontent.com/rust-lang/rust/master/RELEASES.md";
pub const STATIC_DIST_BASE_URL: &str = "https://static.rust-lang.org/dist";
```

## Development

### Requirements

- Rust 1.75 or later
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation

```bash
cargo doc --open
```

### Publishing

```bash
cargo publish
```

## Changelog

### 0.4.0

- **Breaking Change**: Removed database and config file dependencies
- Library mode is now pure in-memory with zero file-system side effects
- CLI mode simplified to fetch-and-display workflow
- Centralized all constants in `src/constants.rs`
- Improved error messages and documentation
- Added comprehensive inline documentation

### 0.3.0

- Initial public release
- Support for stable, beta, and nightly channels
- GitHub API and RELEASES.md data sources
- Concurrent fetching with semaphore control

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

- [Rust](https://www.rust-lang.org/) - The programming language this tool tracks
- [GitHub API](https://docs.github.com/en/rest) - For stable release data
- [rust-lang.org](https://static.rust-lang.org/) - For beta/nightly channel data