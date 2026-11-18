use std::path::Path;

use rusqlite::{Connection, MAIN_DB, OpenFlags, backup::Progress};

use super::migrations::{current_version, migrate, stored_version};
use super::{Database, DbError, DbResult};

/// Type of the (unused) progress callback of SQLite's backup functions.
type NoProgress = fn(Progress);

impl Database {
    /// Writes a consistent copy of the whole database to `destination`.
    /// An existing file at that place is overwritten.
    pub fn backup_to(&self, destination: &Path) -> DbResult<()> {
        if destination.exists() {
            // SQLite would otherwise try to merge into the old file.
            std::fs::remove_file(destination)?;
        }
        self.conn.backup(MAIN_DB, destination, None::<NoProgress>)?;
        Ok(())
    }

    /// Replaces the complete content of the database by the backup file
    /// `source`. The file is checked first; an unusable file leaves the
    /// current data untouched.
    pub fn restore_from(&mut self, source: &Path) -> DbResult<()> {
        check_backup_file(source)?;
        self.conn.restore(MAIN_DB, source, None::<NoProgress>)?;
        // The backup may come from an older program version.
        migrate(&mut self.conn)?;
        Ok(())
    }
}

/// Verifies that a file is an intact database of this application.
fn check_backup_file(path: &Path) -> DbResult<()> {
    let invalid = |reason: &str| DbError::InvalidBackup(reason.to_string());

    // Read-only, so a wrong file is never modified or created.
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| invalid("Die Datei lässt sich nicht öffnen."))?;
    let version = stored_version(&conn)
        .map_err(|_| invalid("Die Datei ist keine SQLite-Datenbank."))?;
    if version == 0 {
        return Err(invalid("Die Datei enthält keine Notenverwaltungs-Daten."));
    }
    if version > current_version() {
        return Err(DbError::NewerSchema { found: version, supported: current_version() });
    }
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|_| invalid("Die Datei ist beschädigt."))?;
    if integrity != "ok" {
        return Err(invalid("Die Datei ist beschädigt."));
    }
    let has_tables: bool = conn.query_row(
        "SELECT COUNT(*) = 4 FROM sqlite_master
         WHERE type = 'table' AND name IN ('semester', 'subject', 'category', 'entry')",
        [],
        |row| row.get(0),
    )?;
    if !has_tables {
        return Err(invalid("Die Datei enthält keine Notenverwaltungs-Daten."));
    }
    Ok(())
}
