use jiff::civil::Date;

use super::{CategoryId, EntryId, Fraction, Grade, Points, fraction};

/// What was achieved in a single performance.
///
/// Being an enum, a value is *either* a grade *or* points. A points entry
/// without maximum or a grade entry with points cannot be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryValue {
    /// A plain grade.
    Grade(Grade),
    /// Achieved points out of a maximum.
    Points {
        achieved: Points,
        max: Points,
        /// Grade the teacher actually gave. Takes precedence over the
        /// grading key when present.
        actual_grade: Option<Grade>,
    },
}

impl EntryValue {
    /// Achieved share of the maximum (1/2 = 50 %); `None` for grade entries
    /// and for a (never stored) maximum of zero.
    pub fn ratio(&self) -> Option<Fraction> {
        match self {
            EntryValue::Points { achieved, max, .. } if max.hundredths() > 0 => {
                Some(fraction(achieved.hundredths(), max.hundredths()))
            }
            _ => None,
        }
    }
}

/// One performance (a test, an exam, a participation grade, …).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: EntryId,
    pub category_id: CategoryId,
    pub date: Date,
    /// Free text, may be empty.
    pub description: String,
    pub value: EntryValue,
}
