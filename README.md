# rs-histver

[![Crates.io](https://img.shields.io/crates/v/rs-histver.svg)](https://crates.io/crates/rs-histver)
[![Documentation](https://docs.rs/rs-histver/badge.svg)](https://docs.rs/rs-histver)
[![License](https://img.shields.io/crates/l/rs-histver.svg)](https://github.com/RyenLee/rs-histver#license)

A CLI tool and library for querying Rust historical release versions with local redb cache. Supports stable, beta, and nightly channels.

## Installation

```bash
cargo install rs-histver
```

Or build from source:

```bash
git clone https://github.com/RyenLee/rs-histver.git
cd rs-histver
cargo build --release
```

## Quick Start

```bash
# Sync stable releases to local cache
rs-histver sync

# Sync full stable history (all releases)
rs-histver sync --full

# Sync nightly releases from the last 30 days
rs-histver sync -c nightly

# Sync beta releases from the last 14 days
rs-histver sync -c beta --days 14

# List cached releases
rs-histver list

# List nightly releases
rs-histver list -c nightly

# Search releases
rs-histver search "1.75"
rs-histver search "2024-06" -c stable

# Show cache statistics
rs-histver info
rs-histver info -c nightly
```

## Commands

### Global Options

| Option | Description |
|--------|-------------|
| `-h, --help` | Show help |
| `-V, --version` | Show version |

### `sync` — Sync release data

Fetch release information from remote and cache to local redb database.

```bash
rs-histver sync [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `-c, --channel <CHANNEL>` | `stable` | Channel: `stable` / `beta` / `nightly` |
| `--full` | — | Use RELEASES.md data source (stable only, full history) |
| `-d, --days <DAYS>` | `30` | Probe recent N days of history (beta/nightly only) |

**Data sources:**

| Channel | Default source | `--full` source |
|---------|---------------|-----------------|
| stable | GitHub Releases API | RELEASES.md (full history) |
| beta | `dist/YYYY-MM-DD/channel-rust-beta.toml` date probing | — |
| nightly | `dist/YYYY-MM-DD/channel-rust-nightly.toml` date probing | — |

### `list` — List cached releases

```bash
rs-histver list [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `-n, --limit <N>` | `20` | Maximum entries to display |
| `-c, --channel <CHANNEL>` | all | Filter by channel |

### `search` — Search releases

Fuzzy match by version number or date.

```bash
rs-histver search <KEYWORD> [OPTIONS]
```

| Argument/Option | Description |
|-----------------|-------------|
| `<KEYWORD>` | Search keyword, e.g. `1.75` or `2024-06` |
| `-c, --channel <CHANNEL>` | Filter by channel |

### `info` — Cache statistics

```bash
rs-histver info [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-c, --channel <CHANNEL>` | Filter by channel |

## Database

The local cache uses [redb](https://github.com/cberner/redb) with per-channel tables for efficient filtering:

- `rs_histver_stable` — key: date, value: version
- `rs_histver_beta` — key: date, value: version
- `rs_histver_nightly` — key: date, value: version

Database file is automatically created at:

```
<executable_directory>/data/rs-histver.redb
```

No configuration file is needed. The `data/` directory is created automatically on first use.

> **Note:** If upgrading from v0.1.x, the database format has changed. The old database will be automatically recreated on first run.

## Project Structure

```
src/
├── lib.rs               # Library entry point (HistVer, Config, public API)
├── main.rs              # CLI entry point
├── cli.rs               # CLI module root
│   └── cli/
│       └── types.rs     # Channel enum, Cli, Commands definitions
├── domain.rs             # Domain module root
│   └── domain/
│       └── release.rs   # RustRelease data model
├── infra.rs              # Infrastructure module root
│   ├── infra/
│   │   ├── config.rs    # Configuration (Config, DatabaseConfig, NetworkConfig)
│   │   ├── database.rs  # Database access layer (Db, redb)
│   │   └── fetcher.rs   # ReleaseFetcher trait + create_fetcher factory
│   └── infra/fetcher/
│       ├── http.rs      # Shared HTTP client & TOML utilities
│       ├── stable.rs    # StableFetcher (GitHub API / RELEASES.md)
│       ├── beta.rs      # BetaFetcher (channel TOML probing)
│       └── nightly.rs   # NightlyFetcher (channel TOML probing)
└── app.rs                # Application module root
    └── app/
        ├── handler.rs   # App struct + business logic
        └── display.rs   # Table formatting utilities
```

## API Documentation

Full API documentation is available on [docs.rs](https://docs.rs/rs-histver).

### Usage as a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rs-histver = "0.2"
```

#### Basic Usage

```rust
use rs_histver::{HistVer, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Default: database at <exe_dir>/data/rs-histver.redb
    let hv = HistVer::new(Config::new())?;

    // Fetch and cache releases
    let releases = hv.fetch_releases("stable", false, 30).await?;
    hv.store_releases(&releases)?;

    // Query local cache
    let all = hv.list_releases(None)?;
    let stable = hv.list_releases(Some("stable"))?;
    let results = hv.search_releases("1.75", None)?;
    let count = hv.count_releases(None)?;

    Ok(())
}
```

#### Custom Database Path

```rust
use rs_histver::{HistVer, Config};

// Use a custom database location
let hv = HistVer::new(Config::with_db_path("/path/to/my-cache.redb"))?;
```

## Design Patterns

- **Strategy Pattern**: `ReleaseFetcher` trait — each channel implements its own fetch logic
- **Factory Method**: `create_fetcher()` — creates the appropriate fetcher based on channel name

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
