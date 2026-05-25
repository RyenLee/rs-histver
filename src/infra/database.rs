use anyhow::{Context, Result};
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};

use super::Config;
use crate::domain::RustRelease;

/// Database access layer backed by redb.
///
/// Uses per-channel tables for efficient filtering:
/// - `{prefix}_stable` — key: date, value: version
/// - `{prefix}_beta`   — key: date, value: version
/// - `{prefix}_nightly`— key: date, value: version
///
/// This design eliminates JSON serialization overhead and enables
/// table-level channel filtering without scanning all records.
pub struct Db {
    db: Database,
    table_prefix: String,
}

impl Db {
    /// Open or create the database at the path derived from `config`.
    ///
    /// Creates parent directories if needed. If the existing database has an
    /// incompatible file format version, it will be recreated automatically.
    /// Ensures all three channel tables exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened, created, or if table creation fails.
    pub fn open(config: &Config) -> Result<Self> {
        let db_path = config.db_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = match Database::create(db_path) {
            Ok(db) => db,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("file format version") {
                    eprintln!(
                        "Warning: Incompatible database format, recreating: {}",
                        db_path.display()
                    );
                    let _ = std::fs::remove_file(db_path);
                    Database::create(db_path).with_context(|| {
                        format!("Failed to create database: {}", db_path.display())
                    })?
                } else {
                    return Err(e).with_context(|| {
                        format!("Failed to open database: {}", db_path.display())
                    });
                }
            }
        };

        // Create all channel tables
        let write_txn = db.begin_write()?;
        for table_name in config.database.all_table_names() {
            let table_def: TableDefinition<&str, &str> = TableDefinition::new(&table_name);
            write_txn.open_table(table_def)?;
        }
        write_txn.commit()?;

        Ok(Self {
            db,
            table_prefix: config.database.table_prefix.clone(),
        })
    }

    /// Get the table name for a given channel.
    fn table_name_for(&self, channel: &str) -> String {
        format!("{}_{}", self.table_prefix, channel)
    }

    /// Insert or update all release records into the appropriate channel table.
    ///
    /// All releases in a batch should belong to the same channel.
    /// Key = date, Value = version.
    /// Returns the number of records written.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write transaction fails.
    pub fn upsert_all(&self, releases: &[RustRelease]) -> Result<u64> {
        if releases.is_empty() {
            return Ok(0);
        }

        // Group by channel and write each group to its table
        let mut count = 0u64;
        let write_txn = self.db.begin_write()?;

        for channel in &["stable", "beta", "nightly"] {
            let channel_releases: Vec<_> =
                releases.iter().filter(|r| r.channel == *channel).collect();

            if channel_releases.is_empty() {
                continue;
            }

            let table_name = self.table_name_for(channel);
            let mut table =
                write_txn.open_table::<&str, &str>(TableDefinition::new(&table_name))?;

            for r in &channel_releases {
                table.insert(r.date.as_str(), r.version.as_str())?;
                count += 1;
            }
        }

        write_txn.commit()?;
        Ok(count)
    }

    /// Read and deserialize all records from a specific channel table.
    fn read_channel(&self, channel: &str) -> Result<Vec<RustRelease>> {
        let table_name = self.table_name_for(channel);
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table::<&str, &str>(TableDefinition::new(&table_name))?;

        let mut releases = Vec::new();
        for entry in table.iter()? {
            let (key, val) = entry?;
            releases.push(RustRelease {
                version: val.value().to_string(),
                date: key.value().to_string(),
                channel: channel.to_string(),
            });
        }
        Ok(releases)
    }

    /// Read all releases, optionally filtered by channel.
    fn read_all(&self, channel: Option<&str>) -> Result<Vec<RustRelease>> {
        let mut releases = Vec::new();

        match channel {
            Some(ch) => {
                releases = self.read_channel(ch)?;
            }
            None => {
                for ch in &["stable", "beta", "nightly"] {
                    releases.extend(self.read_channel(ch)?);
                }
            }
        }

        Ok(releases)
    }

    /// List all releases, optionally filtered by channel (sorted by date desc)
    ///
    /// # Errors
    ///
    /// Returns an error if the database read fails.
    pub fn list_all(&self, channel: Option<&str>) -> Result<Vec<RustRelease>> {
        let mut releases = self.read_all(channel)?;
        releases.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(releases)
    }

    /// Search releases by keyword, optionally filtered by channel.
    ///
    /// Filters before sorting to avoid unnecessary sort on non-matching records.
    ///
    /// # Errors
    ///
    /// Returns an error if the database read fails.
    pub fn search(&self, keyword: &str, channel: Option<&str>) -> Result<Vec<RustRelease>> {
        let all = self.read_all(channel)?;
        let kw = keyword.to_lowercase();
        let mut results: Vec<RustRelease> = all
            .into_iter()
            .filter(|r| r.version.to_lowercase().contains(&kw) || r.date.contains(&kw))
            .collect();
        results.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(results)
    }

    /// Count releases, optionally filtered by channel.
    /// Uses `table.len()` directly — no deserialization needed.
    ///
    /// # Errors
    ///
    /// Returns an error if the database read fails.
    pub fn count(&self, channel: Option<&str>) -> Result<u64> {
        if let Some(ch) = channel {
            let table_name = self.table_name_for(ch);
            let read_txn = self.db.begin_read()?;
            let table = read_txn.open_table::<&str, &str>(TableDefinition::new(&table_name))?;
            Ok(table.len()?)
        } else {
            let mut total = 0u64;
            for ch in &["stable", "beta", "nightly"] {
                let table_name = self.table_name_for(ch);
                let read_txn = self.db.begin_read()?;
                let table =
                    read_txn.open_table::<&str, &str>(TableDefinition::new(&table_name))?;
                total += table.len()?;
            }
            Ok(total)
        }
    }
}
