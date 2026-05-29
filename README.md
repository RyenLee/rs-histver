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
| `--data-dir <PATH>` | Data directory for database files |
| `--db-file <NAME>` | Database filename (enables shared mode) |
| `--table-prefix <PREFIX>` | Prefix for database table names (default: `rs_histver`) |
| `--timeout <SECONDS>` | HTTP request timeout (default: `15`) |
| `--max-concurrency <N>` | Maximum concurrent HTTP requests (default: `10`) |

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

### Default path

Database file is automatically created at:

```
<executable_directory>/data/rs-histver.redb
```

No configuration file is needed. The `data/` directory is created automatically on first use.

### Database Modes

rs-histver supports two database modes, determined by the `shared` flag in [`DatabaseConfig`](https://docs.rs/rs-histver/latest/rs_histver/struct.DatabaseConfig.html):

| Mode | `shared` | Trigger Condition | Database File | Table Prefix |
|------|----------|-------------------|---------------|--------------|
| **Standalone** (default) | `false` | No `db_file` configured | `<data_dir>/rs-histver.redb` | `rs_histver` |
| **Shared** | `true` | `db_file` is set | `<data_dir>/<db_file>` | Custom (recommended) |

#### Standalone Mode

Creates a dedicated `rs-histver.redb` file. This is the default behavior and requires no special configuration.

**Triggered when:**
- Using `Config::new()` — default configuration
- Using `Config::with_db_path(path)` — custom path, always standalone
- Using `ConfigBuilder` without `db_file()` — even with custom `data_dir()`
- CLI without `--db-file` argument

#### Shared Mode

Opens the host project's existing redb database file, using `table_prefix` to avoid table name collisions.

**Triggered when:**
- `db_file` is set via `ConfigBuilder::db_file()` method
- CLI with `--db-file` argument

**Important:** When `db_file` is set but `table_prefix` remains at its default value (`rs_histver`), a warning is printed to stderr to prevent potential table name collisions with other applications using the same database.

### Automatic Fallback Behavior

Shared mode includes intelligent error recovery:

| Scenario | Behavior |
|----------|----------|
| Target file is not a valid redb database | Falls back to standalone mode, creates `rs-histver.redb` in same directory |
| Database format version incompatible (`UpgradeRequired`) | Deletes old file and recreates (standalone mode) |
| Database file corrupted (`StorageError::Corrupted`) | Falls back to standalone mode |
| Table does not exist during read | Returns empty result (not an error) |

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
rs-histver = "0.2.1"
```

#### Standalone Usage

```rust
use rs_histver::{HistVer, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Default: database at <exe_dir>/data/rs-histver.redb
    let hv = HistVer::new(Config::new())?;

    // Or: custom database path
    // let hv = HistVer::new(Config::with_db_path("/path/to/my-cache.redb"))?;

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

#### Embedded in Host Project

When embedded in a host project (e.g. Tauri app), the database path should be determined by the host. Use `ConfigBuilder` to pass configuration programmatically.

**Standalone mode (default):**

Creates a dedicated `rs-histver.redb` file in `data_dir`.

```rust
use rs_histver::{HistVer, ConfigBuilder};

let hv = HistVer::new(
    ConfigBuilder::new()
        .data_dir(app_data_dir)          // creates <data_dir>/rs-histver.redb
        .build()?
)?;
```

**Shared mode (host uses redb):**

When the host project also uses redb, you can share the same database file. Configure `db_file` to point to the host's redb file, and `table_prefix` to avoid table name collisions.

```rust
use rs_histver::{HistVer, ConfigBuilder};

let hv = HistVer::new(
    ConfigBuilder::new()
        .data_dir(app_data_dir)
        .db_file("myapp.redb")          // shared mode
        .table_prefix("myapp_histver")
        .build()?
)?;
```

If `db_file` points to a non-redb file, it automatically falls back to standalone mode (creates `rs-histver.redb` in the same directory).

**Priority order:**

```
Programmatic override (highest) → hardcoded defaults (lowest)
```

- `db_path()` overrides everything (complete file path, always standalone mode)
- `db_file()` enables shared mode, combined with `data_dir()`
- `data_dir()` alone → standalone mode: `<data_dir>/rs-histver.redb`
- `table_prefix()` / `timeout()` / `max_concurrency()` override defaults

## Design Patterns

- **Strategy Pattern**: `ReleaseFetcher` trait — each channel implements its own fetch logic
- **Factory Method**: `create_fetcher()` — creates the appropriate fetcher based on channel name

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
