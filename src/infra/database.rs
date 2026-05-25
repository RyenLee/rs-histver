use anyhow::{Context, Result};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use super::Config;
use crate::domain::RustRelease;

/// Database access layer backed by redb.
///
/// Provides CRUD operations for `RustRelease` records, persisted in a local
/// redb key-value store.
pub struct Db {
    db: Database,
    table_name: String,
}

impl Db {
    /// Open or create the database at the path derived from `config`.
    ///
    /// Creates parent directories if needed. If the existing database has an
    /// incompatible file format version, it will be recreated automatically.
    pub fn open(config: &Config) -> Result<Self> {
        let db_path = config.db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = match Database::create(&db_path) {
            Ok(db) => db,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("file format version") {
                    eprintln!(
                        "Warning: Incompatible database format, recreating: {}",
                        db_path.display()
                    );
                    let _ = std::fs::remove_file(&db_path);
                    Database::create(&db_path)
                        .with_context(|| format!("Failed to create database: {}", db_path.display()))?
                } else {
                    return Err(e).with_context(|| format!("Failed to open database: {}", db_path.display()));
                }
            }
        };

        let table_name = config.database.table_name.clone();
        let table_def: TableDefinition<&str, &str> = TableDefinition::new(&table_name);
        let write_txn = db.begin_write()?;
        write_txn.open_table(table_def)?;
        write_txn.commit()?;

        Ok(Self { db, table_name })
    }

    /// Insert or update all release records into the database.
    ///
    /// Returns the number of records written.
    pub fn upsert_all(&self, releases: &[RustRelease]) -> Result<u64> {
        let write_txn = self.db.begin_write()?;
        let mut count = 0u64;
        {
            let mut table =
                write_txn.open_table::<&str, &str>(TableDefinition::new(&self.table_name))?;
            for r in releases {
                let key = r.db_key();
                let json = serde_json::to_string(r)?;
                table.insert(key.as_str(), json.as_str())?;
                count += 1;
            }
        }
        write_txn.commit()?;
        Ok(count)
    }

    /// List all releases, optionally filtered by channel
    pub fn list_all(&self, channel: Option<&str>) -> Result<Vec<RustRelease>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table::<&str, &str>(TableDefinition::new(&self.table_name))?;
        let mut releases: Vec<RustRelease> = Vec::new();
        for entry in table.iter()? {
            let (_, val) = entry?;
            let r: RustRelease = serde_json::from_str(val.value())?;
            if let Some(ch) = channel {
                if r.channel != ch {
                    continue;
                }
            }
            releases.push(r);
        }
        releases.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(releases)
    }

    /// Search releases by keyword, optionally filtered by channel
    pub fn search(&self, keyword: &str, channel: Option<&str>) -> Result<Vec<RustRelease>> {
        let all = self.list_all(channel)?;
        let kw = keyword.to_lowercase();
        Ok(all
            .into_iter()
            .filter(|r| r.version.to_lowercase().contains(&kw) || r.date.contains(&kw))
            .collect())
    }

    /// Count releases, optionally filtered by channel
    pub fn count(&self, channel: Option<&str>) -> Result<u64> {
        let all = self.list_all(channel)?;
        Ok(all.len() as u64)
    }
}
