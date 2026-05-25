# rs-histver

A CLI tool to query Rust historical release versions with local redb cache. Supports stable, beta, and nightly channels.

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
| `-C, --config <PATH>` | Path to config file |
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

## Configuration

The tool searches for `config.toml` in the following order:

1. Path specified by `-C` CLI argument
2. `config.toml` in the current working directory
3. `config.toml` in the executable's directory
4. Default configuration if none found

### Configuration Options

```toml
[database]
# Database file path
# - Empty: use system default path
# - Relative: based on executable directory
# - Absolute: used as-is
path = ""

# Database table name
table_name = "rust_releases"

[network]
# HTTP request timeout (seconds)
timeout = 15

# Max concurrent connections (for beta/nightly probing)
max_concurrency = 10

# Request User-Agent
user_agent = "rs-histver/0.1"
```

### Default Database Path

| OS | Path |
|----|------|
| Windows | `%LOCALAPPDATA%\rs-histver\releases.redb` |
| Linux | `~/.local/share/rs-histver/releases.redb` |
| macOS | `~/Library/Application Support/rs-histver/releases.redb` |

## Project Structure

```
src/
├── main.rs              # Entry point
├── cli.rs               # CLI module root
│   └── cli/
│       └── types.rs     # Channel enum, Cli, Commands definitions
├── domain.rs             # Domain module root
│   └── domain/
│       └── release.rs   # RustRelease data model
├── infra.rs              # Infrastructure module root
│   ├── infra/
│   │   ├── config.rs    # Configuration loading (Config, DatabaseConfig, NetworkConfig)
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

## Design Patterns

- **Strategy Pattern**: `ReleaseFetcher` trait — each channel implements its own fetch logic
- **Factory Method**: `create_fetcher()` — creates the appropriate fetcher based on channel name

## API Documentation

Full API documentation is available on [docs.rs](https://docs.rs/rs-histver) after publishing, or generate locally:

```bash
cargo doc --open
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
