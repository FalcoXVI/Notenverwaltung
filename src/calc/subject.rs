use super::category::{CategoryResult, evaluate_category};
use crate::domain::{Fraction, Grade, SubjectData, fraction};

/// The overall standing in a subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overall {
    /// Weighted average of the category grades.
    pub average: Fraction,
    /// The average rounded up to a whole grade.
    pub grade: Grade,
    /// `true` if any contributing category is only estimated.
    pub estimated: bool,
}

/// Result of evaluating a whole subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectResult {
    /// `None` while no category has a counting entry.
    pub overall: Option<Overall>,
    /// One result per category, in the order of `SubjectData::categories`.
    pub categories: Vec<CategoryResult>,
}

/// Calculates all category grades of a subject and its overall grade.
pub fn evaluate_subject(data: &SubjectData) -> SubjectResult {
    let categories: Vec<CategoryResult> = data
        .categories
        .iter()
        .map(|c| evaluate_category(&c.category, &c.entries, &data.subject.key))
        .collect();
    let weights = data.categories.iter().map(|c| c.category.weight_percent);
    let overall = weighted_average(weights.zip(&categories));
    SubjectResult { overall, categories }
}

/// Combines category results into the subject average.
///
/// Categories without a grade drop out. Dividing by the sum of the
/// *remaining* weights distributes their share proportionally.
pub(super) fn weighted_average<'a>(
    parts: impl Iterator<Item = (u32, &'a CategoryResult)>,
) -> Option<Overall> {
    let mut weighted_sum = fraction(0, 1);
    let mut weight_sum = 0_i64;
    let mut estimated = false;

    for (weight, result) in parts {
        let Some(graded) = result.graded else { continue };
        if weight == 0 {
            continue;
        }
        weighted_sum += graded.grade * i64::from(weight);
        weight_sum += i64::from(weight);
        estimated |= graded.estimated;
    }

    if weight_sum == 0 {
        return None;
    }
    let average = weighted_sum / weight_sum;
    Some(Overall { average, grade: round_up(average), estimated })
}

/// Rounds an average up to the next whole grade: 4.0 stays a 4, anything
/// above 4.0 becomes a 5.
pub fn round_up(average: Fraction) -> Grade {
    let number = average.ceil().to_integer().clamp(1, 5);
    Grade::from_number(number).expect("clamped to 1..=5")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::test_support::{category, grade, points, subject_data};
    use crate::domain::{CategoryKind, GradingKey};

    const UNKNOWN: GradingKey = GradingKey {
        thresholds: GradingKey::DEFAULT_THRESHOLDS,
        known: false,
    };
    const KNOWN: GradingKey = GradingKey { known: true, ..UNKNOWN };

    #[test]
    fn rounds_up_at_every_boundary() {
        assert_eq!(round_up(fraction(1, 1)), Grade::One);
        assert_eq!(round_up(fraction(101, 100)), Grade::Two);
        assert_eq!(round_up(fraction(3, 1)), Grade::Three);
        assert_eq!(round_up(fraction(4, 1)), Grade::Four);
        assert_eq!(round_up(fraction(401, 100)), Grade::Five);
        assert_eq!(round_up(fraction(4_000_001, 1_000_000)), Grade::Five);
        assert_eq!(round_up(fraction(5, 1)), Grade::Five);
    }

    #[test]
    fn exactly_four_is_positive_slightly_above_is_not() {
        // 4.0: three categories that each have a 4. With f64 the weights
        // 33/33/34 would be a classic source of rounding noise.
        let exactly_four = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 33, None), vec![grade(4)]),
            (category(CategoryKind::Grade, 33, None), vec![grade(4)]),
            (category(CategoryKind::Grade, 34, None), vec![grade(3), grade(5)]),
        ]);
        let overall = evaluate_subject(&exactly_four).overall.unwrap();
        assert_eq!(overall.average, fraction(4, 1));
        assert_eq!(overall.grade, Grade::Four);

        // 4.01: 99 % weight on a 4, 1 % on a 5.
        let just_above = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 99, None), vec![grade(4)]),
            (category(CategoryKind::Grade, 1, None), vec![grade(5)]),
        ]);
        let overall = evaluate_subject(&just_above).overall.unwrap();
        assert_eq!(overall.average, fraction(401, 100));
        assert_eq!(overall.grade, Grade::Five);
    }

    #[test]
    fn one_point_zero_one_is_already_a_two() {
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 99, None), vec![grade(1)]),
            (category(CategoryKind::Grade, 1, None), vec![grade(2)]),
        ]);
        let overall = evaluate_subject(&data).overall.unwrap();
        assert_eq!(overall.average, fraction(101, 100));
        assert_eq!(overall.grade, Grade::Two);
    }

    #[test]
    fn weights_are_applied() {
        // 60 % * 2 + 40 % * 4 = 2.8 -> 3
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 60, None), vec![grade(2)]),
            (category(CategoryKind::Grade, 40, None), vec![grade(4)]),
        ]);
        let overall = evaluate_subject(&data).overall.unwrap();
        assert_eq!(overall.average, fraction(14, 5));
        assert_eq!(overall.grade, Grade::Three);
    }

    #[test]
    fn empty_categories_pass_their_weight_on_proportionally() {
        // 50/30/20 with the 20 % category empty: (50*2 + 30*4) / 80 = 2.75
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 50, None), vec![grade(2)]),
            (category(CategoryKind::Grade, 30, None), vec![grade(4)]),
            (category(CategoryKind::Points, 20, None), vec![]),
        ]);
        let result = evaluate_subject(&data);
        let overall = result.overall.unwrap();
        assert_eq!(overall.average, fraction(11, 4));
        assert_eq!(overall.grade, Grade::Three);
        assert_eq!(result.categories[2].graded, None);
    }

    #[test]
    fn a_single_filled_category_decides_alone() {
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 10, None), vec![grade(3)]),
            (category(CategoryKind::Points, 90, None), vec![]),
        ]);
        let overall = evaluate_subject(&data).overall.unwrap();
        assert_eq!(overall.average, fraction(3, 1));
    }

    #[test]
    fn subject_without_entries_has_no_grade() {
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 100, None), vec![]),
        ]);
        assert_eq!(evaluate_subject(&data).overall, None);
    }

    #[test]
    fn estimate_flag_propagates_from_category_to_subject() {
        let categories = || vec![
            (category(CategoryKind::Grade, 50, None), vec![grade(2)]),
            (category(CategoryKind::Points, 50, None), vec![points(40, 48)]),
        ];
        let unknown = evaluate_subject(&subject_data(UNKNOWN, categories()));
        assert!(unknown.overall.unwrap().estimated);
        // Entering the real key makes the same data exact.
        let known = evaluate_subject(&subject_data(KNOWN, categories()));
        assert!(!known.overall.unwrap().estimated);
    }

    #[test]
    fn subject_with_only_exact_parts_is_exact_despite_unknown_key() {
        // Grade entries and a clear 5 do not depend on the assumed key.
        let data = subject_data(UNKNOWN, vec![
            (category(CategoryKind::Grade, 50, None), vec![grade(2)]),
            (category(CategoryKind::Points, 50, None), vec![points(10, 48)]),
        ]);
        assert!(!evaluate_subject(&data).overall.unwrap().estimated);
    }

    #[test]
    fn real_key_changes_the_result() {
        // 40/48 = 83.3 %: a 2 by the default key, a 3 with a stricter key.
        let strict = GradingKey { thresholds: [9500, 8500, 7000, 5000], known: true };
        let categories = || vec![
            (category(CategoryKind::Points, 100, None), vec![points(40, 48)]),
        ];
        let default = evaluate_subject(&subject_data(UNKNOWN, categories()));
        assert_eq!(default.overall.unwrap().grade, Grade::Two);
        let real = evaluate_subject(&subject_data(strict, categories()));
        assert_eq!(real.overall.unwrap().grade, Grade::Three);
    }
}
