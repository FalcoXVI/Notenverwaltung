use std::cmp::Reverse;

use super::grading::entry_grade;
use crate::domain::{CategoryKind, EntryValue, GradingKey, fraction};

/// Decides for every entry whether it counts towards the category grade.
///
/// Entries of the wrong kind never count. With `best_n = Some(n)` only the
/// `n` best remaining entries count. "Better" means: better grade (actual
/// grade if entered, otherwise via the grading key), and on equal grades
/// the higher percentage. On a full tie the earlier entry wins.
pub fn select_counted(
    kind: CategoryKind,
    best_n: Option<u32>,
    values: &[EntryValue],
    key: &GradingKey,
) -> Vec<bool> {
    let mut eligible: Vec<usize> = (0..values.len())
        .filter(|&index| kind.accepts(&values[index]))
        .collect();

    if let Some(n) = best_n {
        // `sort_by_key` is stable, which gives the "earlier entry wins" rule.
        eligible.sort_by_key(|&index| {
            let value = &values[index];
            let ratio = value.ratio().unwrap_or(fraction(0, 1));
            (entry_grade(value, key).grade, Reverse(ratio))
        });
        eligible.truncate(n as usize);
    }

    let mut counted = vec![false; values.len()];
    for index in eligible {
        counted[index] = true;
    }
    counted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::test_support::{grade, points, points_actual};

    fn key() -> GradingKey {
        GradingKey::default_unknown()
    }

    #[test]
    fn without_limit_everything_counts() {
        let values = [grade(1), grade(5), grade(3)];
        let counted = select_counted(CategoryKind::Grade, None, &values, &key());
        assert_eq!(counted, [true, true, true]);
    }

    #[test]
    fn of_two_tests_only_the_better_counts() {
        let values = [points(20, 48), points(40, 48)];
        let counted = select_counted(CategoryKind::Points, Some(1), &values, &key());
        assert_eq!(counted, [false, true]);
    }

    #[test]
    fn best_two_of_three_grades() {
        let values = [grade(4), grade(2), grade(3)];
        let counted = select_counted(CategoryKind::Grade, Some(2), &values, &key());
        assert_eq!(counted, [false, true, true]);
    }

    #[test]
    fn actual_grade_beats_higher_percentage() {
        // 30/48 with an actual 2 is better than 40/48 (a 2 by key) only on
        // a tie-break, so make the key grade worse: 35/48 is a 3 by key.
        let values = [points(35, 48), points_actual(30, 48, 2)];
        let counted = select_counted(CategoryKind::Points, Some(1), &values, &key());
        assert_eq!(counted, [false, true]);
    }

    #[test]
    fn equal_grades_are_decided_by_percentage() {
        // Both are a 4 by the default key; 28/48 is the higher percentage.
        let values = [points(25, 48), points(28, 48)];
        let counted = select_counted(CategoryKind::Points, Some(1), &values, &key());
        assert_eq!(counted, [false, true]);
    }

    #[test]
    fn full_tie_keeps_the_earlier_entry() {
        let values = [grade(3), grade(3)];
        let counted = select_counted(CategoryKind::Grade, Some(1), &values, &key());
        assert_eq!(counted, [true, false]);
    }

    #[test]
    fn limit_larger_than_entry_count_counts_all() {
        let values = [grade(3), grade(4)];
        let counted = select_counted(CategoryKind::Grade, Some(5), &values, &key());
        assert_eq!(counted, [true, true]);
    }

    #[test]
    fn entries_of_the_wrong_kind_never_count() {
        let values = [grade(1), points(40, 48)];
        let counted = select_counted(CategoryKind::Points, None, &values, &key());
        assert_eq!(counted, [false, true]);
    }
}
