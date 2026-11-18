use rusqlite::Connection;

use super::{DbError, DbResult};

/// All schema migrations in order. Entry `i` upgrades a database from
/// version `i` to version `i + 1`. To change the schema, append a new
/// entry; never modify an existing one.
const MIGRATIONS: &[&str] = &[include_str!("schema_v1.sql")];

/// The schema version this program works with.
pub fn current_version() -> i64 {
    MIGRATIONS.len() as i64
}

/// Reads the schema version stored in the database file (0 = empty file).
pub fn stored_version(conn: &Connection) -> DbResult<i64> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Brings the schema up to date by running all missing migrations.
///
/// Each migration runs in its own transaction together with the version
/// bump, so a failure can never leave a half-migrated database behind.
pub fn migrate(conn: &mut Connection) -> DbResult<()> {
    let found = stored_version(conn)?;
    if found > current_version() {
        return Err(DbError::NewerSchema { found, supported: current_version() });
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(found as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", index as i64 + 1)?;
        tx.commit()?;
    }
    Ok(())
}
