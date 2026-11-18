use rusqlite::params;

use super::{Database, DbResult};
use crate::domain::{Semester, SemesterId};

impl Database {
    /// Creates an empty semester.
    pub fn insert_semester(&mut self, number: u32, label: &str) -> DbResult<SemesterId> {
        self.conn.execute(
            "INSERT INTO semester (number, label) VALUES (?1, ?2)",
            params![number, label],
        )?;
        Ok(SemesterId(self.conn.last_insert_rowid()))
    }

    /// Creates a semester as a copy of `source`: subjects, categories,
    /// weights and grading keys are taken over; entries, report grades and
    /// exam grades are not.
    pub fn copy_semester(&mut self, source: SemesterId, number: u32, label: &str) -> DbResult<SemesterId> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO semester (number, label) VALUES (?1, ?2)",
            params![number, label],
        )?;
        let new_semester = tx.last_insert_rowid();

        let subject_ids: Vec<i64> = tx
            .prepare("SELECT id FROM subject WHERE semester_id = ?1 ORDER BY id")?
            .query_map([source.0], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;

        for old_subject in subject_ids {
            tx.execute(
                "INSERT INTO subject (semester_id, name, key_known,
                                      threshold_1, threshold_2, threshold_3, threshold_4)
                 SELECT ?1, name, key_known, threshold_1, threshold_2, threshold_3, threshold_4
                 FROM subject WHERE id = ?2",
                params![new_semester, old_subject],
            )?;
            let new_subject = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO category (subject_id, name, weight_percent, kind, best_n, sort_order)
                 SELECT ?1, name, weight_percent, kind, best_n, sort_order
                 FROM category WHERE subject_id = ?2",
                params![new_subject, old_subject],
            )?;
        }
        tx.commit()?;
        Ok(SemesterId(new_semester))
    }

    /// Changes number and label of a semester.
    pub fn update_semester(&mut self, semester: &Semester) -> DbResult<()> {
        self.conn.execute(
            "UPDATE semester SET number = ?1, label = ?2 WHERE id = ?3",
            params![semester.number, semester.label, semester.id.0],
        )?;
        Ok(())
    }

    /// Deletes a semester including all its subjects and entries.
    pub fn delete_semester(&mut self, id: SemesterId) -> DbResult<()> {
        self.conn.execute("DELETE FROM semester WHERE id = ?1", [id.0])?;
        Ok(())
    }
}
