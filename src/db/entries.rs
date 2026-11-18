use jiff::civil::Date;
use rusqlite::params;

use super::row_mapping::{DATE_FORMAT, value_to_db};
use super::{Database, DbResult};
use crate::domain::{CategoryId, Entry, EntryId, EntryValue};

impl Database {
    /// Adds an entry to a category.
    pub fn insert_entry(
        &mut self,
        category_id: CategoryId,
        date: Date,
        description: &str,
        value: &EntryValue,
    ) -> DbResult<EntryId> {
        let (grade, achieved, max, actual) = value_to_db(value);
        self.conn.execute(
            "INSERT INTO entry (category_id, date, description, grade,
                                points_achieved, points_max, actual_grade)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                category_id.0,
                date.strftime(DATE_FORMAT).to_string(),
                description,
                grade,
                achieved,
                max,
                actual
            ],
        )?;
        Ok(EntryId(self.conn.last_insert_rowid()))
    }

    /// Overwrites date, description and value of an existing entry.
    pub fn update_entry(&mut self, entry: &Entry) -> DbResult<()> {
        let (grade, achieved, max, actual) = value_to_db(&entry.value);
        self.conn.execute(
            "UPDATE entry SET date = ?1, description = ?2, grade = ?3,
                    points_achieved = ?4, points_max = ?5, actual_grade = ?6
             WHERE id = ?7",
            params![
                entry.date.strftime(DATE_FORMAT).to_string(),
                entry.description,
                grade,
                achieved,
                max,
                actual,
                entry.id.0
            ],
        )?;
        Ok(())
    }

    /// Deletes an entry.
    pub fn delete_entry(&mut self, id: EntryId) -> DbResult<()> {
        self.conn.execute("DELETE FROM entry WHERE id = ?1", [id.0])?;
        Ok(())
    }
}
