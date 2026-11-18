use rusqlite::{Transaction, params};

use super::row_mapping::{kind_to_db, optional_grade_to_db};
use super::{CategoryDraft, Database, DbResult, SubjectDraft};
use crate::domain::{Grade, SemesterId, SubjectId};

impl Database {
    /// Creates or updates a subject together with its categories.
    ///
    /// The stored categories are replaced by the draft's list: categories
    /// missing from the draft are deleted (including their entries), the
    /// others are updated or inserted. Everything happens in one
    /// transaction, so the subject is never saved halfway.
    pub fn save_subject(&mut self, semester_id: SemesterId, draft: &SubjectDraft) -> DbResult<SubjectId> {
        let tx = self.conn.transaction()?;
        let [t1, t2, t3, t4] = draft.key.thresholds;

        let subject_id = match draft.id {
            Some(id) => {
                tx.execute(
                    "UPDATE subject SET name = ?1, key_known = ?2, threshold_1 = ?3,
                            threshold_2 = ?4, threshold_3 = ?5, threshold_4 = ?6
                     WHERE id = ?7",
                    params![draft.name, draft.key.known, t1, t2, t3, t4, id.0],
                )?;
                id.0
            }
            None => {
                tx.execute(
                    "INSERT INTO subject (semester_id, name, key_known, threshold_1,
                                          threshold_2, threshold_3, threshold_4)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![semester_id.0, draft.name, draft.key.known, t1, t2, t3, t4],
                )?;
                tx.last_insert_rowid()
            }
        };

        save_categories(&tx, subject_id, &draft.categories)?;
        tx.commit()?;
        Ok(SubjectId(subject_id))
    }

    /// Stores the report grade and the semester exam result of a subject.
    pub fn set_final_grades(
        &mut self,
        id: SubjectId,
        report_grade: Option<Grade>,
        exam_grade: Option<Grade>,
    ) -> DbResult<()> {
        self.conn.execute(
            "UPDATE subject SET report_grade = ?1, exam_grade = ?2 WHERE id = ?3",
            params![optional_grade_to_db(report_grade), optional_grade_to_db(exam_grade), id.0],
        )?;
        Ok(())
    }

    /// Deletes a subject including its categories and entries.
    pub fn delete_subject(&mut self, id: SubjectId) -> DbResult<()> {
        self.conn.execute("DELETE FROM subject WHERE id = ?1", [id.0])?;
        Ok(())
    }
}

/// Makes the stored categories of a subject equal to `drafts`.
fn save_categories(tx: &Transaction<'_>, subject_id: i64, drafts: &[CategoryDraft]) -> DbResult<()> {
    // Delete what is no longer in the list. Ids are numbers from our own
    // structs, so building the list into the SQL text is safe.
    let kept: Vec<String> = drafts
        .iter()
        .filter_map(|draft| draft.id)
        .map(|id| id.0.to_string())
        .collect();
    tx.execute(
        &format!(
            "DELETE FROM category WHERE subject_id = ?1 AND id NOT IN ({})",
            kept.join(",")
        ),
        [subject_id],
    )?;

    for (position, draft) in drafts.iter().enumerate() {
        let kind = kind_to_db(draft.kind);
        let position = position as i64;
        match draft.id {
            Some(id) => tx.execute(
                "UPDATE category SET name = ?1, weight_percent = ?2, kind = ?3,
                        best_n = ?4, sort_order = ?5
                 WHERE id = ?6 AND subject_id = ?7",
                params![draft.name, draft.weight_percent, kind, draft.best_n, position, id.0, subject_id],
            )?,
            None => tx.execute(
                "INSERT INTO category (subject_id, name, weight_percent, kind, best_n, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![subject_id, draft.name, draft.weight_percent, kind, draft.best_n, position],
            )?,
        };
    }
    Ok(())
}
