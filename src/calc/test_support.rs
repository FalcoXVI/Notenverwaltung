//! Small builders that keep the unit tests readable.

use crate::domain::{
    Category, CategoryData, CategoryId, CategoryKind, Entry, EntryId, EntryValue, Grade,
    GradingKey, Points, SemesterId, Subject, SubjectData, SubjectId,
};

/// A grade entry value; panics on numbers outside 1..=5.
pub fn grade(number: i64) -> EntryValue {
    EntryValue::Grade(Grade::from_number(number).unwrap())
}

/// A points entry value with whole points and no actual grade.
pub fn points(achieved: i64, max: i64) -> EntryValue {
    EntryValue::Points {
        achieved: Points::from_hundredths(achieved * 100),
        max: Points::from_hundredths(max * 100),
        actual_grade: None,
    }
}

/// A points entry value for which the teacher's grade is known.
pub fn points_actual(achieved: i64, max: i64, actual: i64) -> EntryValue {
    EntryValue::Points {
        achieved: Points::from_hundredths(achieved * 100),
        max: Points::from_hundredths(max * 100),
        actual_grade: Grade::from_number(actual),
    }
}

/// A category with the given settings and placeholder ids.
pub fn category(kind: CategoryKind, weight_percent: u32, best_n: Option<u32>) -> Category {
    Category {
        id: CategoryId(0),
        subject_id: SubjectId(0),
        name: String::new(),
        weight_percent,
        kind,
        best_n,
    }
}

/// A subject made of categories and the values of their entries.
pub fn subject_data(key: GradingKey, categories: Vec<(Category, Vec<EntryValue>)>) -> SubjectData {
    let date = jiff::civil::date(2026, 11, 17);
    SubjectData {
        subject: Subject {
            id: SubjectId(0),
            semester_id: SemesterId(0),
            name: String::new(),
            key,
            report_grade: None,
            exam_grade: None,
        },
        categories: categories
            .into_iter()
            .map(|(category, values)| CategoryData {
                entries: values
                    .into_iter()
                    .map(|value| Entry {
                        id: EntryId(0),
                        category_id: category.id,
                        date,
                        description: String::new(),
                        value,
                    })
                    .collect(),
                category,
            })
            .collect(),
    }
}
