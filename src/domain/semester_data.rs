use super::{Semester, SubjectData};

/// A semester with all of its subjects, categories and entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemesterData {
    pub semester: Semester,
    pub subjects: Vec<SubjectData>,
}
