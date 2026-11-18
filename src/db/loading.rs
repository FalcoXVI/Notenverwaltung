use super::row_mapping::{date_from_db, kind_from_db, optional_grade_from_db, value_from_db};
use super::{Database, DbResult};
use crate::domain::{
    Category, CategoryData, CategoryId, Entry, EntryId, GradingKey, Semester, SemesterData,
    SemesterId, Subject, SubjectData, SubjectId,
};

impl Database {
    /// Loads the complete content of the database: every semester with its
    /// subjects, categories and entries.
    ///
    /// The amount of data is tiny (a few hundred rows over a whole school
    /// career), so the GUI simply reloads everything after each change
    /// instead of keeping a second copy in sync.
    pub fn load_all(&self) -> DbResult<Vec<SemesterData>> {
        let semesters = self.load_semesters()?;
        semesters
            .into_iter()
            .map(|semester| {
                let subjects = self.load_subjects(semester.id)?;
                Ok(SemesterData { semester, subjects })
            })
            .collect()
    }

    fn load_semesters(&self) -> DbResult<Vec<Semester>> {
        let mut statement = self
            .conn
            .prepare_cached("SELECT id, number, label FROM semester ORDER BY number, id")?;
        let rows = statement.query_map([], |row| {
            Ok(Semester {
                id: SemesterId(row.get(0)?),
                number: row.get(1)?,
                label: row.get(2)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    fn load_subjects(&self, semester_id: SemesterId) -> DbResult<Vec<SubjectData>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT id, name, key_known, threshold_1, threshold_2, threshold_3, threshold_4,
                    report_grade, exam_grade
             FROM subject WHERE semester_id = ?1 ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([semester_id.0], |row| {
            Ok(Subject {
                id: SubjectId(row.get(0)?),
                semester_id,
                name: row.get(1)?,
                key: GradingKey {
                    known: row.get(2)?,
                    thresholds: [row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?],
                },
                report_grade: optional_grade_from_db(row.get(7)?)?,
                exam_grade: optional_grade_from_db(row.get(8)?)?,
            })
        })?;
        let subjects: Vec<Subject> = rows.collect::<rusqlite::Result<_>>()?;

        subjects
            .into_iter()
            .map(|subject| {
                let categories = self.load_categories(subject.id)?;
                Ok(SubjectData { subject, categories })
            })
            .collect()
    }

    fn load_categories(&self, subject_id: SubjectId) -> DbResult<Vec<CategoryData>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT id, name, weight_percent, kind, best_n
             FROM category WHERE subject_id = ?1 ORDER BY sort_order, id",
        )?;
        let rows = statement.query_map([subject_id.0], |row| {
            Ok(Category {
                id: CategoryId(row.get(0)?),
                subject_id,
                name: row.get(1)?,
                weight_percent: row.get(2)?,
                kind: kind_from_db(&row.get::<_, String>(3)?)?,
                best_n: row.get(4)?,
            })
        })?;
        let categories: Vec<Category> = rows.collect::<rusqlite::Result<_>>()?;

        categories
            .into_iter()
            .map(|category| {
                let entries = self.load_entries(category.id)?;
                Ok(CategoryData { category, entries })
            })
            .collect()
    }

    fn load_entries(&self, category_id: CategoryId) -> DbResult<Vec<Entry>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT id, date, description, grade, points_achieved, points_max, actual_grade
             FROM entry WHERE category_id = ?1 ORDER BY date, id",
        )?;
        let rows = statement.query_map([category_id.0], |row| {
            Ok(Entry {
                id: EntryId(row.get(0)?),
                category_id,
                date: date_from_db(&row.get::<_, String>(1)?)?,
                description: row.get(2)?,
                value: value_from_db((row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?))?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
