//! Persistence in a SQLite file. All SQL of the application lives here.

mod auto_backup;
mod backup;
mod database;
mod entries;
mod error;
mod loading;
mod migrations;
mod paths;
mod row_mapping;
mod semesters;
mod settings;
mod subject_draft;
mod subjects;
#[cfg(test)]
mod tests;

pub use auto_backup::create_auto_backup;
pub use database::Database;
pub use error::{DbError, DbResult};
pub use paths::{backup_dir, database_file};
pub use subject_draft::{CategoryDraft, SubjectDraft};
