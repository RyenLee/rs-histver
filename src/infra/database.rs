use anyhow::{Context, Result};
use redb::{
    Database, DatabaseError, ReadableDatabase, ReadableTable, ReadableTableMetadata, StorageError,
    TableDefinition, TableError,
};

use super::config::DEFAULT_DB_FILENAME;
use super::Config;
use crate::domain::RustRelease;

/// Database access layer backed by redb.
///
/// Uses per-channel tables for efficient filtering:
/// - `{prefix}_stable` — key: date, value: version
/// - `{prefix}_beta`   — key: date, value: version
/// - `{prefix}_nightly`— key: date, value: version
///
/// Supports two modes:
/// - **Standalone** (default): creates a dedicated `rs-histver.redb` file
/// - **Shared**: opens the host project's existing redb database, using
///   `table_prefix` to avoid name collisions
///
/// In shared mode, if the target file is not a valid redb database, it
/// automatically falls back to standalone mode.
pub struct Db {
    db: Database,
    table_prefix: String,
}

impl Db {
    /// Open or create the database based on [`Config`].
    ///
    /// Creates parent directories if needed. In standalone mode, an incompatible
    /// database file is recreated. In shared mode, a non-redb file triggers an
    /// automatic fallback to standalone mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened, created, or if table
    /// creation fails.
    pub fn open(config: &Config) -> Result<Self> {
        let db_path = config.db_path();
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let db = if config.database.shared {
            Self::open_shared(db_path)?
        } else {
            Self::open_standalone(db_path)?
        };

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

    fn open_shared(db_path: &std::path::Path) -> Result<Database> {
        if db_path.exists() {
            match Database::open(db_path) {
                Ok(db) => Ok(db),
                Err(e) => {
                    let db_err: anyhow::Error = e.into();
                    let is_redb_error = db_err.downcast_ref::<DatabaseError>().is_some_and(|err| {
                        matches!(
                            err,
                            DatabaseError::UpgradeRequired(_)
                                | DatabaseError::Storage(StorageError::Corrupted(_))
                        )
                    });
                    if is_redb_error {
                        eprintln!(
                            "Warning: '{}' is not a valid redb database. \
                             Falling back to standalone mode.",
                            db_path.display()
                        );
                    } else {
                        eprintln!(
                            "Warning: '{}' could not be opened ({}). \
                             Falling back to standalone mode.",
                            db_path.display(),
                            db_err
                        );
                    }
                    let standalone_path = db_path.with_file_name(DEFAULT_DB_FILENAME);
                    Self::open_standalone(&standalone_path)
                }
            }
        } else {
            Database::create(db_path).with_context(|| {
                format!("Failed to create shared database: {}", db_path.display())
            })?;
            let db = Database::open(db_path).context(format!(
                "Failed to open shared database: {}",
                db_path.display()
            ))?;
            Ok(db)
        }
    }

    fn open_standalone(db_path: &std::path::Path) -> Result<Database> {
        match Database::create(db_path) {
            Ok(db) => Ok(db),
            Err(e) => {
                let anyhow_err: anyhow::Error = e.into();
                let needs_recreate = anyhow_err
                    .downcast_ref::<DatabaseError>()
                    .is_some_and(|err| matches!(err, DatabaseError::UpgradeRequired(_)));
                if needs_recreate {
                    eprintln!(
                        "Warning: Incompatible database format, recreating: {}",
                        db_path.display()
                    );
                    let _ = std::fs::remove_file(db_path);
                    Ok(Database::create(db_path).with_context(|| {
                        format!("Failed to create database: {}", db_path.display())
                    })?)
                } else {
                    Err(anyhow_err)
                        .context(format!("Failed to open database: {}", db_path.display()))
                }
            }
        }
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
        let table = match read_txn.open_table::<&str, &str>(TableDefinition::new(&table_name)) {
            Ok(t) => t,
            Err(e) => {
                let anyhow_err: anyhow::Error = e.into();
                let table_not_found = anyhow_err
                    .downcast_ref::<TableError>()
                    .is_some_and(|err| matches!(err, TableError::TableDoesNotExist(_)));
                if table_not_found {
                    return Ok(Vec::new());
                }
                return Err(anyhow_err).context(format!("Failed to open table: {}", table_name));
            }
        };

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
            let count = match read_txn.open_table::<&str, &str>(TableDefinition::new(&table_name)) {
                Ok(table) => table.len()?,
                Err(e) => {
                    let anyhow_err: anyhow::Error = e.into();
                    let table_not_found = anyhow_err
                        .downcast_ref::<TableError>()
                        .is_some_and(|err| matches!(err, TableError::TableDoesNotExist(_)));
                    if table_not_found {
                        0
                    } else {
                        return Err(anyhow_err)
                            .context(format!("Failed to open table: {}", table_name));
                    }
                }
            };
            Ok(count)
        } else {
            let mut total = 0u64;
            for ch in &["stable", "beta", "nightly"] {
                let table_name = self.table_name_for(ch);
                let read_txn = self.db.begin_read()?;
                let count =
                    match read_txn.open_table::<&str, &str>(TableDefinition::new(&table_name)) {
                        Ok(table) => table.len()?,
                        Err(e) => {
                            let anyhow_err: anyhow::Error = e.into();
                            let table_not_found = anyhow_err
                                .downcast_ref::<TableError>()
                                .is_some_and(|err| matches!(err, TableError::TableDoesNotExist(_)));
                            if table_not_found {
                                0
                            } else {
                                return Err(anyhow_err)
                                    .context(format!("Failed to open table: {}", table_name));
                            }
                        }
                    };
                total += count;
            }
            Ok(total)
        }
    }
}
