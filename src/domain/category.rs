use super::{CategoryId, EntryValue, SubjectId};

/// How the entries of a category are recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryKind {
    /// Each entry is a grade 1–5.
    Grade,
    /// Each entry is "achieved of maximum" points.
    Points,
}

impl CategoryKind {
    /// Whether an entry value belongs into a category of this kind.
    pub fn accepts(self, value: &EntryValue) -> bool {
        matches!(
            (self, value),
            (CategoryKind::Grade, EntryValue::Grade(_))
                | (CategoryKind::Points, EntryValue::Points { .. })
        )
    }

    /// German name shown in the GUI.
    pub fn label(self) -> &'static str {
        match self {
            CategoryKind::Grade => "Note 1–5",
            CategoryKind::Points => "Punkte",
        }
    }
}

/// A group of performances within a subject, e.g. "Schularbeit" or "Test".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub id: CategoryId,
    pub subject_id: SubjectId,
    pub name: String,
    /// Share of the subject grade in percent (all categories sum to 100).
    pub weight_percent: u32,
    pub kind: CategoryKind,
    /// `Some(n)`: only the best `n` entries count. `None`: all count.
    pub best_n: Option<u32>,
}
