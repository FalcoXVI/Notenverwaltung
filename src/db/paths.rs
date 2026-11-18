use std::path::PathBuf;

/// Folder name below `%APPDATA%`.
const APP_FOLDER: &str = "notenverwaltung";

/// Directory holding all data of the application.
fn data_dir() -> PathBuf {
    // APPDATA is always set on Windows; the fallback keeps the program
    // usable (next to the working directory) in unusual environments.
    let base = std::env::var_os("APPDATA").map_or_else(|| PathBuf::from("."), PathBuf::from);
    base.join(APP_FOLDER)
}

/// Path of the database file, `%APPDATA%\notenverwaltung\noten.db`.
pub fn database_file() -> PathBuf {
    data_dir().join("noten.db")
}

/// Directory for the automatic backups.
pub fn backup_dir() -> PathBuf {
    data_dir().join("backups")
}
