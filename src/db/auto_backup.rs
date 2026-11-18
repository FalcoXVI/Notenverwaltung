use std::path::{Path, PathBuf};

use jiff::Zoned;

use super::{Database, DbResult};

/// Number of automatic backups that are kept.
const KEEP_BACKUPS: usize = 10;

const PREFIX: &str = "noten-";
const SUFFIX: &str = ".db";

/// Writes a timestamped backup into `dir` and deletes the oldest automatic
/// backups so that at most [`KEEP_BACKUPS`] remain. Returns the new file.
pub fn create_auto_backup(db: &Database, dir: &Path) -> DbResult<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let timestamp = Zoned::now().strftime("%Y%m%d-%H%M%S");
    let target = dir.join(format!("{PREFIX}{timestamp}{SUFFIX}"));
    db.backup_to(&target)?;
    prune_old_backups(dir)?;
    Ok(target)
}

/// Deletes all but the newest automatic backups.
fn prune_old_backups(dir: &Path) -> DbResult<()> {
    let mut backups: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| is_auto_backup(path))
        .collect();
    // The timestamp in the name sorts chronologically.
    backups.sort();
    let surplus = backups.len().saturating_sub(KEEP_BACKUPS);
    for old in &backups[..surplus] {
        std::fs::remove_file(old)?;
    }
    Ok(())
}

/// Only files that follow our naming scheme are ever deleted.
fn is_auto_backup(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(PREFIX) && name.ends_with(SUFFIX))
}
