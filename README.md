# rs-histver

[![Crates.io](https://img.shields.io/crates/v/rs-histver.svg)](https://crates.io/crates/rs-histver)
[![Documentation](https://docs.rs/rs-histver/badge.svg)](https://docs.rs/rs-histver)
[![License](https://img.shields.io/crates/l/rs-histver.svg)](https://github.com/RyenLee/rs-histver#license)

A library for querying Rust historical release versions (stable / beta / nightly) via the GitHub Releases API and rust-lang.org distribution server.

**As a library**: one function call, pure in-memory result, zero file-system side effects.
**As a CLI binary**: local redb cache for offline queries, full terminal UI.

## Library vs CLI

```toml
# Library only — no redb / clap dependency
rs-histver = "0.3"

# Library + CLI binary
rs-histver = { version = "0.3", features = ["cli"] }
```

### Usage as a Library

```rust
use rs_histver::{fetch_releases, FetchOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Stable releases via GitHub API (default: 30-day history)
    let releases = fetch_releases("stable", FetchOptions::default()).await?;

    // Full history from RELEASES.md
    let all = fetch_releases("stable", FetchOptions::new().full_history(true)).await?;

    // Recent 7 days of nightly builds
    let recent = fetch_releases("nightly", FetchOptions::new().probe_days(7)).await?;

    for r in &releases {
        println!("{} ({}) [{}]", r.version, r.date, r.channel);
    }
    Ok(())
}
```

#### `FetchOptions` — all fields are optional

| Method | Default | Description |
|--------|---------|-------------|
| `full_history(bool)` | `false` | Stable: use RELEASES.md instead of GitHub API |
| `probe_days(u32)` | `30` | Beta/nightly: how many recent days to probe |
| `timeout(Duration)` | `15s` | HTTP request timeout |
| `max_concurrency(usize)` | `10` | Maximum concurrent HTTP requests |
| `user_agent(string)` | `rs-histver/{ver}` | HTTP User-Agent header |

#### Library API

| Item | Type | Description |
|------|------|-------------|
| `fetch_releases(channel, opts)` | `async fn` | Fetch `Vec<RustRelease>` from remote |
| `FetchOptions` | `struct` | Query configuration |
| `RustRelease` | `struct` | `{ version, date, channel }` |
| `create_fetcher(channel, full, days)` | `fn` | Low-level: create a typed fetcher |
| `ReleaseFetcher` | `trait` | Strategy trait; implement to extend |

```rust
// Low-level API — use create_fetcher + ReleaseFetcher trait directly
use rs_histver::{create_fetcher, ReleaseFetcher, NetworkConfig};

let fetcher = create_fetcher("stable", false, 30)?;
let network = NetworkConfig { timeout: 15, max_concurrency: 10, user_agent: "my-app/1.0".into() };
let releases = fetcher.fetch(&network).await?;
```

## CLI Installation

```bash
cargo install rs-histver
```

Or build from source:

```bash
git clone https://github.com/RyenLee/rs-histver.git
cd rs-histver
cargo build --release --features cli
```

## CLI Quick Start

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

# Search releases
rs-histver search "1.75"

# Show cache statistics
rs-histver info
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `sync [-c CHANNEL] [--full] [-d DAYS]` | Fetch remote data and cache locally |
| `list [-c CHANNEL] [-n LIMIT]` | List cached releases |
| `search <KEYWORD> [-c CHANNEL]` | Fuzzy search by version or date |
| `info [-c CHANNEL]` | Show cache entry counts |

### Global Options

| Option | Description |
|--------|-------------|
| `--data-dir <PATH>` | Data directory for database files |
| `--db-file <NAME>` | Database filename |
| `--table-prefix <PREFIX>` | Table name prefix (default: `rs_histver`) |
| `--timeout <SECONDS>` | HTTP timeout (default: `15`) |
| `--max-concurrency <N>` | Max concurrent requests (default: `10`) |

## Data Sources

| Channel | Default source | `--full` source |
|---------|---------------|----------------|
| stable | GitHub Releases API | RELEASES.md (full history) |
| beta | `dist/{date}/channel-rust-beta.toml` probing | — |
| nightly | `dist/{date}/channel-rust-nightly.toml` probing | — |

## Database (CLI only)

The CLI caches data in [redb](https://github.com/cberner/redb):

```
<executable_directory>/data/rs-histver.redb
```

Three per-channel tables (`{prefix}_stable`, `{prefix}_beta`, `{prefix}_nightly`) store
key-value pairs where key = date and value = version string.

### Database Modes (CLI)

| Mode | Trigger | File |
|------|---------|------|
| **Standalone** (default) | No `--db-file` | `<data_dir>/rs-histver.redb` |
| **Shared** | `--db-file` set | `<data_dir>/<db_file>` |

When `db_file` points to a non-redb file or the file is locked by another process,
the CLI automatically falls back to standalone mode.

## Design Patterns

- **Strategy Pattern**: `ReleaseFetcher` trait — each channel implements its own fetch logic
- **Factory Method**: `create_fetcher()` — creates the appropriate fetcher by channel name

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
