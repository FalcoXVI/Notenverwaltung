use super::best_n::select_counted;
use super::grading::entry_grade;
use crate::domain::{Category, CategoryKind, Entry, EntryValue, Fraction, GradingKey, fraction};

/// A calculated grade that may have decimals (e.g. an average of 2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Graded {
    pub grade: Fraction,
    /// `true` if the value depends on an assumed grading key.
    pub estimated: bool,
}

/// Result of evaluating one category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryResult {
    /// `None` if no entry counts (the category then drops out of the
    /// subject average).
    pub graded: Option<Graded>,
    /// For each entry, in input order: does it count? Entries that do not
    /// count are shown struck through.
    pub counted: Vec<bool>,
}

/// Calculates the grade of a category from its entries.
pub fn evaluate_category(category: &Category, entries: &[Entry], key: &GradingKey) -> CategoryResult {
    let values: Vec<EntryValue> = entries.iter().map(|entry| entry.value).collect();
    evaluate_values(category.kind, category.best_n, &values, key)
}

/// Same as [`evaluate_category`], but on bare values. The what-if
/// calculation uses this to try out hypothetical entries.
pub(super) fn evaluate_values(
    kind: CategoryKind,
    best_n: Option<u32>,
    values: &[EntryValue],
    key: &GradingKey,
) -> CategoryResult {
    let counted = select_counted(kind, best_n, values, key);
    let counting: Vec<EntryValue> = values
        .iter()
        .zip(&counted)
        .filter(|(_, counts)| **counts)
        .map(|(value, _)| *value)
        .collect();

    let graded = if counting.is_empty() {
        None
    } else if has_actual_grade(&counting) || kind == CategoryKind::Grade {
        Some(average_of_entry_grades(&counting, key))
    } else {
        Some(grade_of_point_totals(&counting, key))
    };
    CategoryResult { graded, counted }
}

/// Whether the teacher's actual grade is known for at least one entry.
fn has_actual_grade(values: &[EntryValue]) -> bool {
    values
        .iter()
        .any(|value| matches!(value, EntryValue::Points { actual_grade: Some(_), .. }))
}

/// Converts every entry into a grade and averages those grades.
fn average_of_entry_grades(values: &[EntryValue], key: &GradingKey) -> Graded {
    let grades: Vec<_> = values.iter().map(|value| entry_grade(value, key)).collect();
    let sum: i64 = grades.iter().map(|g| g.grade.number()).sum();
    Graded {
        grade: fraction(sum, grades.len() as i64),
        estimated: grades.iter().any(|g| g.estimated),
    }
}

/// Adds up achieved and maximum points and applies the grading key to the
/// overall percentage.
fn grade_of_point_totals(values: &[EntryValue], key: &GradingKey) -> Graded {
    let (mut achieved_sum, mut max_sum) = (0, 0);
    for value in values {
        if let EntryValue::Points { achieved, max, .. } = value {
            achieved_sum += achieved.hundredths();
            max_sum += max.hundredths();
        }
    }
    let grade = key.grade_for(fraction(achieved_sum, max_sum.max(1)));
    Graded {
        grade: fraction(grade.number(), 1),
        estimated: key.is_estimate(grade),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::test_support::{grade, points, points_actual};

    const UNKNOWN: GradingKey = GradingKey {
        thresholds: GradingKey::DEFAULT_THRESHOLDS,
        known: false,
    };
    const KNOWN: GradingKey = GradingKey { known: true, ..UNKNOWN };

    fn eval(kind: CategoryKind, best_n: Option<u32>, values: &[EntryValue], key: &GradingKey) -> Option<Graded> {
        evaluate_values(kind, best_n, values, key).graded
    }

    fn exact(numerator: i64, denominator: i64) -> Option<Graded> {
        Some(Graded { grade: fraction(numerator, denominator), estimated: false })
    }

    fn estimated(numerator: i64, denominator: i64) -> Option<Graded> {
        Some(Graded { grade: fraction(numerator, denominator), estimated: true })
    }

    #[test]
    fn empty_category_has_no_grade() {
        assert_eq!(eval(CategoryKind::Grade, None, &[], &KNOWN), None);
        assert_eq!(eval(CategoryKind::Points, None, &[], &KNOWN), None);
    }

    #[test]
    fn grade_category_is_the_plain_average() {
        let values = [grade(1), grade(2), grade(2)];
        assert_eq!(eval(CategoryKind::Grade, None, &values, &UNKNOWN), exact(5, 3));
    }

    #[test]
    fn points_are_summed_before_applying_the_key() {
        // 2 tests with 48 points each: 48 of 96 is exactly 50 % -> negative.
        let half = [points(30, 48), points(18, 48)];
        assert_eq!(eval(CategoryKind::Points, None, &half, &KNOWN), exact(5, 1));
        // One point more is positive.
        let above = [points(30, 48), points(19, 48)];
        assert_eq!(eval(CategoryKind::Points, None, &above, &KNOWN), exact(4, 1));
    }

    #[test]
    fn half_points_are_exact() {
        // 48.5 of 96 is just above 50 %.
        let values = [EntryValue::Points {
            achieved: crate::domain::Points::from_hundredths(4850),
            max: crate::domain::Points::from_hundredths(9600),
            actual_grade: None,
        }];
        assert_eq!(eval(CategoryKind::Points, None, &values, &KNOWN), exact(4, 1));
    }

    #[test]
    fn positive_key_grade_is_estimated_while_key_unknown() {
        let values = [points(40, 48)];
        assert_eq!(eval(CategoryKind::Points, None, &values, &UNKNOWN), estimated(2, 1));
        assert_eq!(eval(CategoryKind::Points, None, &values, &KNOWN), exact(2, 1));
    }

    #[test]
    fn a_five_is_exact_even_with_unknown_key() {
        let values = [points(10, 48)];
        assert_eq!(eval(CategoryKind::Points, None, &values, &UNKNOWN), exact(5, 1));
    }

    #[test]
    fn actual_grades_switch_to_per_entry_average() {
        // Entry 1: teacher gave a 3. Entry 2: 46/48 is a 1 by key.
        // Per-entry average: (3 + 1) / 2 = 2.
        let values = [points_actual(40, 48, 3), points(46, 48)];
        assert_eq!(eval(CategoryKind::Points, None, &values, &KNOWN), exact(2, 1));
        // The second entry still depends on the key -> estimated if unknown.
        assert_eq!(eval(CategoryKind::Points, None, &values, &UNKNOWN), estimated(2, 1));
    }

    #[test]
    fn only_actual_grades_are_exact_with_unknown_key() {
        let values = [points_actual(40, 48, 3), points_actual(20, 48, 4)];
        assert_eq!(eval(CategoryKind::Points, None, &values, &UNKNOWN), exact(7, 2));
    }

    #[test]
    fn best_n_drops_the_worse_test() {
        let values = [points(20, 48), points(40, 48)];
        let result = evaluate_values(CategoryKind::Points, Some(1), &values, &KNOWN);
        assert_eq!(result.counted, [false, true]);
        assert_eq!(result.graded, exact(2, 1));
    }

    #[test]
    fn dropped_actual_grade_does_not_switch_the_mode() {
        // The entry with the actual grade is dropped, so the remaining
        // entries are summed as usual: 80 of 96 -> 2.
        let values = [points_actual(10, 48, 5), points(40, 48), points(40, 48)];
        let result = evaluate_values(CategoryKind::Points, Some(2), &values, &KNOWN);
        assert_eq!(result.counted, [false, true, true]);
        assert_eq!(result.graded, exact(2, 1));
    }
}
