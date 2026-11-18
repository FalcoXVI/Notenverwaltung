use crate::domain::SubjectId;

/// The page currently shown in the main area of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// All subjects of the selected semester.
    Overview,
    /// Categories, entries and what-if for one subject.
    Subject(SubjectId),
    /// Negative subjects across all semesters that are not repaired yet.
    OpenNegative,
    /// Traffic light limits, CSV export, backup and restore.
    Settings,
}
