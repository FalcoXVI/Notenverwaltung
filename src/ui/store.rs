use std::path::Path;

use super::command::Command;
use crate::calc::TrafficThresholds;
use crate::db::{Database, DbError, DbResult, backup_dir, create_auto_backup, database_file};
use crate::domain::{SemesterData, SemesterId, SubjectData, SubjectId};
use crate::export::export_csv_file;

/// The data shown by the GUI: the open database plus a copy of its whole
/// content in memory. Views only read from the copy; after every change
/// the copy is reloaded, so screen and file can never drift apart.
pub struct Store {
    db: Database,
    semesters: Vec<SemesterData>,
    thresholds: TrafficThresholds,
}

impl Store {
    /// Opens the database in `%APPDATA%` and loads its content. The second
    /// value is a warning if the automatic backup could not be written.
    pub fn open() -> DbResult<(Store, Option<DbError>)> {
        let (db, warning) = Database::open_with_auto_backup(&database_file(), &backup_dir())?;
        let mut store = Store {
            db,
            semesters: Vec::new(),
            thresholds: TrafficThresholds::default(),
        };
        store.reload()?;
        Ok((store, warning))
    }

    /// All semesters, ordered by number.
    pub fn semesters(&self) -> &[SemesterData] {
        &self.semesters
    }

    /// The traffic light limits from the settings.
    pub fn thresholds(&self) -> &TrafficThresholds {
        &self.thresholds
    }

    /// Looks up a semester by id.
    pub fn semester(&self, id: SemesterId) -> Option<&SemesterData> {
        self.semesters.iter().find(|s| s.semester.id == id)
    }

    /// Looks up a subject by id, together with the semester it belongs to.
    pub fn subject(&self, id: SubjectId) -> Option<(&SemesterData, &SubjectData)> {
        self.semesters.iter().find_map(|semester| {
            let subject = semester.subjects.iter().find(|s| s.subject.id == id)?;
            Some((semester, subject))
        })
    }

    /// Applies a change to the database and reloads the in-memory copy.
    pub fn execute(&mut self, command: Command) -> DbResult<()> {
        let result = self.write(command);
        // Reload even after an error so the screen shows what is really stored.
        self.reload()?;
        result
    }

    /// Writes a backup of the database to a file chosen by the user.
    pub fn backup_to(&self, path: &Path) -> DbResult<()> {
        self.db.backup_to(path)
    }

    /// Exports all grades as CSV.
    pub fn export_csv(&self, path: &Path) -> std::io::Result<()> {
        export_csv_file(path, &self.semesters)
    }

    fn write(&mut self, command: Command) -> DbResult<()> {
        let db = &mut self.db;
        match command {
            Command::CreateSemester { number, label, copy_from: Some(source) } => {
                db.copy_semester(source, number, &label)?;
            }
            Command::CreateSemester { number, label, copy_from: None } => {
                db.insert_semester(number, &label)?;
            }
            Command::UpdateSemester(semester) => db.update_semester(&semester)?,
            Command::DeleteSemester(id) => db.delete_semester(id)?,
            Command::SaveSubject { semester_id, draft } => {
                db.save_subject(semester_id, &draft)?;
            }
            Command::DeleteSubject(id) => db.delete_subject(id)?,
            Command::SetFinalGrades { subject_id, report_grade, exam_grade } => {
                db.set_final_grades(subject_id, report_grade, exam_grade)?;
            }
            Command::AddEntry { category_id, date, description, value } => {
                db.insert_entry(category_id, date, &description, &value)?;
            }
            Command::UpdateEntry(entry) => db.update_entry(&entry)?,
            Command::DeleteEntry(id) => db.delete_entry(id)?,
            Command::SaveTrafficThresholds(thresholds) => db.save_traffic_thresholds(&thresholds)?,
            Command::Restore(path) => {
                // Keep the current state so that a restore can be undone.
                create_auto_backup(db, &backup_dir())?;
                db.restore_from(&path)?;
            }
        }
        Ok(())
    }

    fn reload(&mut self) -> DbResult<()> {
        self.semesters = self.db.load_all()?;
        self.thresholds = self.db.load_traffic_thresholds()?;
        Ok(())
    }
}
