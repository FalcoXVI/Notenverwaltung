use super::category::{CategoryResult, evaluate_category, evaluate_values};
use super::subject::{Overall, weighted_average};
use crate::domain::{CategoryKind, EntryValue, Grade, Points, SubjectData};

/// What the next performance has to bring to reach a target grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    /// The target is reached even with the worst possible result.
    AlreadySafe,
    /// At least this many points are needed (points categories).
    MinPoints(Points),
    /// This grade or a better one is needed (grade categories).
    WorstGrade(Grade),
    /// Not even the best possible result is enough.
    Unreachable,
}

/// Answer to a what-if question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatIf {
    pub requirement: Requirement,
    /// `true` if the answer depends on an assumed grading key.
    pub estimated: bool,
}

/// Points are tried in steps of half a point.
const POINT_STEP_HUNDREDTHS: usize = 50;

/// Calculates what is needed in the *next* performance of one category so
/// that the subject grade is `target` or better.
///
/// `next_max` is the maximum of the next performance and only matters for
/// points categories. The answer is found by simply trying every possible
/// result from worst to best through the normal calculation, so rules such
/// as "best N count" are respected automatically.
///
/// # Panics
/// Panics if `category_index` is out of range.
pub fn required_for_target(
    data: &SubjectData,
    category_index: usize,
    next_max: Points,
    target: Grade,
) -> WhatIf {
    let key = &data.subject.key;
    let target_category = &data.categories[category_index].category;
    let existing_results: Vec<CategoryResult> = data
        .categories
        .iter()
        .map(|c| evaluate_category(&c.category, &c.entries, key))
        .collect();
    let existing_values: Vec<EntryValue> = data.categories[category_index]
        .entries
        .iter()
        .map(|entry| entry.value)
        .collect();

    // Subject result if `candidate` were added to the chosen category.
    let outcome = |candidate: EntryValue| -> Option<Overall> {
        let mut values = existing_values.clone();
        values.push(candidate);
        let changed = evaluate_values(target_category.kind, target_category.best_n, &values, key);
        let parts = data.categories.iter().enumerate().map(|(index, c)| {
            let result = if index == category_index { &changed } else { &existing_results[index] };
            (c.category.weight_percent, result)
        });
        weighted_average(parts)
    };

    let candidates = candidates_worst_to_best(target_category.kind, next_max);
    let outcomes: Vec<Option<Overall>> = candidates.iter().map(|c| outcome(*c)).collect();
    let reaches = |o: &Option<Overall>| o.is_some_and(|overall| overall.grade <= target);

    let (requirement, decisive) = match outcomes.iter().position(reaches) {
        Some(0) => (Requirement::AlreadySafe, outcomes.first()),
        Some(index) => (requirement_for(&candidates[index]), outcomes.get(index)),
        None => (Requirement::Unreachable, outcomes.last()),
    };
    let estimated = decisive
        .copied()
        .flatten()
        .is_some_and(|overall| overall.estimated);
    WhatIf { requirement, estimated }
}

/// All results the next performance could have, from worst to best.
fn candidates_worst_to_best(kind: CategoryKind, next_max: Points) -> Vec<EntryValue> {
    match kind {
        CategoryKind::Grade => Grade::ALL.iter().rev().map(|g| EntryValue::Grade(*g)).collect(),
        CategoryKind::Points => {
            let max = next_max.hundredths();
            let mut steps: Vec<i64> = (0..=max).step_by(POINT_STEP_HUNDREDTHS).collect();
            // Make sure the full score is tried even for odd maxima (e.g. 12.25).
            if steps.last() != Some(&max) {
                steps.push(max);
            }
            steps
                .into_iter()
                .map(|achieved| EntryValue::Points {
                    achieved: Points::from_hundredths(achieved),
                    max: next_max,
                    actual_grade: None,
                })
                .collect()
        }
    }
}

/// Turns the first sufficient candidate into a requirement.
fn requirement_for(candidate: &EntryValue) -> Requirement {
    match candidate {
        EntryValue::Grade(grade) => Requirement::WorstGrade(*grade),
        EntryValue::Points { achieved, .. } => Requirement::MinPoints(*achieved),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::test_support::{category, grade, points, subject_data};
    use crate::domain::GradingKey;

    const UNKNOWN: GradingKey = GradingKey {
        thresholds: GradingKey::DEFAULT_THRESHOLDS,
        known: false,
    };
    const KNOWN: GradingKey = GradingKey { known: true, ..UNKNOWN };

    fn whole_points(n: i64) -> Points {
        Points::from_hundredths(n * 100)
    }

    fn ask(data: &SubjectData, index: usize, max: i64, target: i64) -> Requirement {
        let target = Grade::from_number(target).unwrap();
        required_for_target(data, index, whole_points(max), target).requirement
    }

    #[test]
    fn points_needed_to_become_positive() {
        // 20/48 so far; the next test has 48 points. Positive means more
        // than 48 of 96 in total, so 28.5 are needed.
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Points, 100, None), vec![points(20, 48)]),
        ]);
        assert_eq!(
            ask(&data, 0, 48, 4),
            Requirement::MinPoints(Points::from_hundredths(2850))
        );
    }

    #[test]
    fn points_needed_for_a_three() {
        // A 3 needs more than 62.5 % of 96 = 60 points in total -> 40.5 more.
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Points, 100, None), vec![points(20, 48)]),
        ]);
        assert_eq!(
            ask(&data, 0, 48, 3),
            Requirement::MinPoints(Points::from_hundredths(4050))
        );
    }

    #[test]
    fn best_n_lets_the_new_test_replace_the_old_one() {
        // Only the better of two tests counts, so the old 20/48 no longer
        // matters: more than 24 of 48 is enough.
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Points, 100, Some(1)), vec![points(20, 48)]),
        ]);
        assert_eq!(
            ask(&data, 0, 48, 4),
            Requirement::MinPoints(Points::from_hundredths(2450))
        );
    }

    #[test]
    fn best_n_makes_a_good_result_safe() {
        // 47/48 is already a 1 and the next test can only replace it by
        // something better.
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Points, 100, Some(1)), vec![points(47, 48)]),
        ]);
        assert_eq!(ask(&data, 0, 48, 1), Requirement::AlreadySafe);
    }

    #[test]
    fn target_can_be_out_of_reach() {
        // 0/48 so far: even 48/48 gives only 50 % in total.
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Points, 100, None), vec![points(0, 48)]),
        ]);
        assert_eq!(ask(&data, 0, 48, 4), Requirement::Unreachable);
    }

    #[test]
    fn grade_category_answers_with_a_grade() {
        // 4 and 5 so far: (4 + 5 + g) / 3 <= 4 needs g <= 3.
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 100, None), vec![grade(4), grade(5)]),
        ]);
        assert_eq!(ask(&data, 0, 0, 4), Requirement::WorstGrade(Grade::Three));
    }

    #[test]
    fn other_categories_are_taken_into_account() {
        // Participation (40 %) stands at 2. The exam category (60 %) is
        // empty. For a 3 overall: 0.4*2 + 0.6*g <= 3 -> g <= 3.67 -> a 3,
        // i.e. more than 62.5 % of 40 points = 25.5 points.
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 40, None), vec![grade(2)]),
            (category(CategoryKind::Points, 60, None), vec![]),
        ]);
        assert_eq!(
            ask(&data, 1, 40, 3),
            Requirement::MinPoints(Points::from_hundredths(2550))
        );
    }

    #[test]
    fn first_entry_in_an_empty_subject() {
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Grade, 100, None), vec![]),
        ]);
        assert_eq!(ask(&data, 0, 0, 2), Requirement::WorstGrade(Grade::Two));
        assert_eq!(ask(&data, 0, 0, 5), Requirement::AlreadySafe);
    }

    #[test]
    fn answer_is_marked_estimated_with_unknown_key() {
        let categories = || vec![
            (category(CategoryKind::Points, 100, None), vec![points(20, 48)]),
        ];
        let three = Grade::Three;
        let unknown = required_for_target(&subject_data(UNKNOWN, categories()), 0, whole_points(48), three);
        assert!(unknown.estimated);
        let known = required_for_target(&subject_data(KNOWN, categories()), 0, whole_points(48), three);
        assert!(!known.estimated);
    }

    #[test]
    fn odd_maximum_still_tries_the_full_score() {
        // Maximum 12.25: a 1 needs more than 87.5 % = 10.72 -> 11 points.
        let data = subject_data(KNOWN, vec![
            (category(CategoryKind::Points, 100, None), vec![]),
        ]);
        let result = required_for_target(&data, 0, Points::from_hundredths(1225), Grade::One);
        assert_eq!(result.requirement, Requirement::MinPoints(whole_points(11)));
    }
}
