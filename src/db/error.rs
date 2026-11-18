use std::fmt;

/// Everything that can go wrong while reading or writing the database.
#[derive(Debug)]
pub enum DbError {
    /// SQLite reported an error.
    Sqlite(rusqlite::Error),
    /// A file or directory operation failed.
    Io(std::io::Error),
    /// The database was written by a newer version of this program.
    NewerSchema { found: i64, supported: i64 },
    /// A file chosen for restoring is not a usable backup.
    InvalidBackup(String),
}

/// Shorthand for results of database operations.
pub type DbResult<T> = Result<T, DbError>;

impl fmt::Display for DbError {
    /// German message, shown to the user as is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Sqlite(error) => write!(f, "Datenbankfehler: {error}"),
            DbError::Io(error) => write!(f, "Dateifehler: {error}"),
            DbError::NewerSchema { found, supported } => write!(
                f,
                "Die Datenbank stammt von einer neueren Programmversion \
                 (Schema {found}, unterstützt wird bis {supported})."
            ),
            DbError::InvalidBackup(reason) => write!(f, "Keine gültige Sicherung: {reason}"),
        }
    }
}

impl std::error::Error for DbError {}

// The two `From` impls let the `?` operator convert library errors into
// `DbError` automatically.

impl From<rusqlite::Error> for DbError {
    fn from(error: rusqlite::Error) -> Self {
        DbError::Sqlite(error)
    }
}

impl From<std::io::Error> for DbError {
    fn from(error: std::io::Error) -> Self {
        DbError::Io(error)
    }
}
