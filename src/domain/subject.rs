use super::{Grade, GradingKey, SemesterId, SubjectId};

/// A subject within one semester.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    pub id: SubjectId,
    pub semester_id: SemesterId,
    pub name: String,
    pub key: GradingKey,
    /// Grade on the report card, entered by hand once it is known.
    pub report_grade: Option<Grade>,
    /// Result of the semester exam taken to repair a negative grade.
    pub exam_grade: Option<Grade>,
}

impl Subject {
    /// The grade that finally stands: the semester exam replaces the
    /// report grade. `None` while no report grade has been entered.
    pub fn final_grade(&self) -> Option<Grade> {
        self.exam_grade.or(self.report_grade)
    }

    /// A subject is "open negative" if the report card shows a 5 that has
    /// not been repaired by a positive semester exam.
    pub fn is_open_negative(&self) -> bool {
        let repaired = self.exam_grade.is_some_and(Grade::is_positive);
        self.report_grade == Some(Grade::Five) && !repaired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject(report: Option<Grade>, exam: Option<Grade>) -> Subject {
        Subject {
            id: SubjectId(1),
            semester_id: SemesterId(1),
            name: "Mathematik".into(),
            key: GradingKey::default_unknown(),
            report_grade: report,
            exam_grade: exam,
        }
    }

    #[test]
    fn exam_replaces_report_grade() {
        assert_eq!(subject(None, None).final_grade(), None);
        assert_eq!(subject(Some(Grade::Three), None).final_grade(), Some(Grade::Three));
        let repaired = subject(Some(Grade::Five), Some(Grade::Four));
        assert_eq!(repaired.final_grade(), Some(Grade::Four));
    }

    #[test]
    fn open_negative_needs_a_five_without_positive_exam() {
        assert!(!subject(None, None).is_open_negative());
        assert!(!subject(Some(Grade::Four), None).is_open_negative());
        assert!(subject(Some(Grade::Five), None).is_open_negative());
        assert!(subject(Some(Grade::Five), Some(Grade::Five)).is_open_negative());
        assert!(!subject(Some(Grade::Five), Some(Grade::Four)).is_open_negative());
    }
}
