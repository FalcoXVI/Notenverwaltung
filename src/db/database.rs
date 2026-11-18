use std::path::Path;

use rusqlite::Connection;

use super::auto_backup::create_auto_backup;
use super::migrations::{current_version, migrate, stored_version};
use super::{DbError, DbResult};

/// An open database. All reading and writing goes through its methods,
/// which are spread over the sibling files (`semesters.rs`, `subjects.rs`,
/// `entries.rs`, …) as separate `impl Database` blocks.
pub struct Database {
    pub(super) conn: Connection,
}

impl Database {
    /// Opens the database file, creating file and parent directory if
    /// needed, and upgrades the schema to the current version.
    pub fn open(path: &Path) -> DbResult<Database> {
        let mut db = Database::connect_file(path)?;
        migrate(&mut db.conn)?;
        Ok(db)
    }

    /// Like [`Database::open`], but first writes an automatic backup of
    /// the existing data into `backup_dir`.
    ///
    /// A failed backup is returned as a warning next to the database. Only
    /// if a schema upgrade is pending the failure is fatal: existing data
    /// is never migrated without a backup.
    pub fn open_with_auto_backup(
        path: &Path,
        backup_dir: &Path,
    ) -> DbResult<(Database, Option<DbError>)> {
        let mut db = Database::connect_file(path)?;
        let version = stored_version(&db.conn)?;
        let mut warning = None;
        // Version 0 is a brand-new file: nothing to back up yet.
        if version > 0 {
            if let Err(error) = create_auto_backup(&db, backup_dir) {
                if version < current_version() {
                    return Err(error);
                }
                warning = Some(error);
            }
        }
        migrate(&mut db.conn)?;
        Ok((db, warning))
    }

    /// Opens an empty database in memory (used by tests).
    pub fn open_in_memory() -> DbResult<Database> {
        let mut db = Database::configure(Connection::open_in_memory()?)?;
        migrate(&mut db.conn)?;
        Ok(db)
    }

    /// Opens the file without touching the schema.
    fn connect_file(path: &Path) -> DbResult<Database> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Database::configure(Connection::open(path)?)
    }

    /// Applies the connection settings.
    fn configure(conn: Connection) -> DbResult<Database> {
        // Without this pragma SQLite ignores `ON DELETE CASCADE`.
        conn.pragma_update(None, "foreign_keys", true)?;
        // Wait for the disk on every commit: slower, but nothing is lost
        // if the computer crashes right after saving.
        conn.pragma_update(None, "synchronous", "FULL")?;
        Ok(Database { conn })
    }
}
