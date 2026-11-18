use std::path::PathBuf;

use jiff::civil::Date;

use crate::calc::TrafficThresholds;
use crate::db::SubjectDraft;
use crate::domain::{CategoryId, Entry, EntryId, EntryValue, Grade, Semester, SemesterId, SubjectId};

/// A change to the stored data. Views only *describe* changes with this
/// enum; [`super::store::Store::execute`] is the single place that
/// actually writes to the database.
#[derive(Debug, Clone)]
pub enum Command {
    CreateSemester {
        number: u32,
        label: String,
        /// Take over subjects and categories from this semester.
        copy_from: Option<SemesterId>,
    },
    UpdateSemester(Semester),
    DeleteSemester(SemesterId),
    SaveSubject {
        semester_id: SemesterId,
        draft: SubjectDraft,
    },
    DeleteSubject(SubjectId),
    SetFinalGrades {
        subject_id: SubjectId,
        report_grade: Option<Grade>,
        exam_grade: Option<Grade>,
    },
    AddEntry {
        category_id: CategoryId,
        date: Date,
        description: String,
        value: EntryValue,
    },
    UpdateEntry(Entry),
    DeleteEntry(EntryId),
    SaveTrafficThresholds(TrafficThresholds),
    /// Replace all data by the content of a backup file.
    Restore(PathBuf),
}

impl Command {
    /// Message for the status bar after the command succeeded.
    pub fn success_message(&self) -> &'static str {
        match self {
            Command::DeleteSemester(_) | Command::DeleteSubject(_) | Command::DeleteEntry(_) => {
                "Gelöscht."
            }
            Command::Restore(_) => "Sicherung wiederhergestellt.",
            _ => "Gespeichert.",
        }
    }
}
