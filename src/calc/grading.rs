use crate::domain::{EntryValue, Grade, GradingKey};

/// The grade of a single entry and whether it is only an estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryGrade {
    pub grade: Grade,
    pub estimated: bool,
}

/// Converts one entry into a grade.
///
/// A grade entry is its own grade. For points the teacher's actual grade
/// wins; without it the grading key is applied to the entry's percentage.
pub fn entry_grade(value: &EntryValue, key: &GradingKey) -> EntryGrade {
    let exact = |grade| EntryGrade { grade, estimated: false };
    match value {
        EntryValue::Grade(grade) => exact(*grade),
        EntryValue::Points { actual_grade: Some(grade), .. } => exact(*grade),
        EntryValue::Points { actual_grade: None, .. } => {
            // A maximum of zero cannot be stored; treat it as nothing achieved.
            let grade = value.ratio().map_or(Grade::Five, |ratio| key.grade_for(ratio));
            EntryGrade { grade, estimated: key.is_estimate(grade) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::test_support::{grade, points, points_actual};

    #[test]
    fn grade_entry_is_exact() {
        let key = GradingKey::default_unknown();
        let result = entry_grade(&grade(2), &key);
        assert_eq!(result, EntryGrade { grade: Grade::Two, estimated: false });
    }

    #[test]
    fn actual_grade_wins_over_key() {
        let key = GradingKey::default_unknown();
        // 40 of 48 would be a 2 by the default key, the teacher gave a 3.
        let result = entry_grade(&points_actual(40, 48, 3), &key);
        assert_eq!(result, EntryGrade { grade: Grade::Three, estimated: false });
    }

    #[test]
    fn key_grade_is_estimated_while_key_unknown() {
        let unknown = GradingKey::default_unknown();
        let known = GradingKey { known: true, ..unknown };
        assert_eq!(
            entry_grade(&points(40, 48), &unknown),
            EntryGrade { grade: Grade::Two, estimated: true }
        );
        assert_eq!(
            entry_grade(&points(40, 48), &known),
            EntryGrade { grade: Grade::Two, estimated: false }
        );
    }
}
