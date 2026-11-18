use super::{Fraction, Grade, fraction};

/// The grading key of a subject: which percentage is needed for which grade.
///
/// Every threshold means "strictly more than": with a threshold of 50 %
/// for a 4, exactly 50 % is still a 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GradingKey {
    /// Thresholds for the grades 1, 2, 3 and 4 in hundredths of a percent
    /// (87.5 % is stored as 8750). The last one is the pass mark.
    pub thresholds: [i64; 4],
    /// `false` while the teacher's real key is unknown and the thresholds
    /// are only an assumption. Grades derived from it are then "estimated".
    pub known: bool,
}

impl GradingKey {
    /// Thresholds used until the real key is entered.
    pub const DEFAULT_THRESHOLDS: [i64; 4] = [8750, 7500, 6250, 5000];

    /// 100 % expressed in hundredths of a percent.
    pub const FULL: i64 = 10_000;

    /// The assumed key for a subject whose real key is not known yet.
    pub fn default_unknown() -> GradingKey {
        GradingKey {
            thresholds: Self::DEFAULT_THRESHOLDS,
            known: false,
        }
    }

    /// The assumed key for a known pass mark: the range between the pass
    /// mark and 100 % is split into four equal steps. A pass mark of 50 %
    /// yields exactly the default thresholds.
    pub fn assumed_for_pass_mark(pass_mark: i64) -> GradingKey {
        let step = (Self::FULL - pass_mark) / 4;
        GradingKey {
            thresholds: [pass_mark + 3 * step, pass_mark + 2 * step, pass_mark + step, pass_mark],
            known: false,
        }
    }

    /// A key is valid if the thresholds lie within 0..100 % and strictly
    /// decrease from grade 1 to grade 4.
    pub fn is_valid(&self) -> bool {
        let in_range = self.thresholds.iter().all(|t| (0..Self::FULL).contains(t));
        let descending = self.thresholds.windows(2).all(|pair| pair[0] > pair[1]);
        in_range && descending
    }

    /// Maps a ratio `achieved / maximum` (1/2 = 50 %) to a grade.
    pub fn grade_for(&self, ratio: Fraction) -> Grade {
        let grades = [Grade::One, Grade::Two, Grade::Three, Grade::Four];
        for (threshold, grade) in self.thresholds.iter().zip(grades) {
            if ratio > fraction(*threshold, Self::FULL) {
                return grade;
            }
        }
        Grade::Five
    }

    /// Whether a grade computed from this key is only an estimate.
    ///
    /// The pass mark is always known, so a 5 is exact even with an unknown
    /// key. Every better grade depends on the assumed thresholds.
    pub fn is_estimate(&self, grade: Grade) -> bool {
        !self.known && grade != Grade::Five
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_key_boundaries_are_exclusive() {
        let key = GradingKey::default_unknown();
        assert_eq!(key.grade_for(fraction(1, 1)), Grade::One);
        assert_eq!(key.grade_for(fraction(875, 1000)), Grade::Two);
        assert_eq!(key.grade_for(fraction(75, 100)), Grade::Three);
        assert_eq!(key.grade_for(fraction(625, 1000)), Grade::Four);
        assert_eq!(key.grade_for(fraction(1, 2)), Grade::Five);
        assert_eq!(key.grade_for(fraction(0, 1)), Grade::Five);
    }

    #[test]
    fn exactly_half_is_negative_one_point_more_is_positive() {
        let key = GradingKey::default_unknown();
        assert_eq!(key.grade_for(fraction(48, 96)), Grade::Five);
        assert_eq!(key.grade_for(fraction(49, 96)), Grade::Four);
    }

    #[test]
    fn assumed_key_is_spread_evenly_above_the_pass_mark() {
        assert_eq!(GradingKey::assumed_for_pass_mark(5000), GradingKey::default_unknown());
        let sixty = GradingKey::assumed_for_pass_mark(6000);
        assert_eq!(sixty.thresholds, [9000, 8000, 7000, 6000]);
        assert!(!sixty.known);
    }

    #[test]
    fn pass_mark_is_configurable() {
        let key = GradingKey {
            thresholds: [9000, 8000, 7000, 6000],
            known: true,
        };
        assert_eq!(key.grade_for(fraction(60, 100)), Grade::Five);
        assert_eq!(key.grade_for(fraction(61, 100)), Grade::Four);
    }

    #[test]
    fn only_a_five_is_exact_with_unknown_key() {
        let unknown = GradingKey::default_unknown();
        assert!(unknown.is_estimate(Grade::Three));
        assert!(unknown.is_estimate(Grade::Four));
        assert!(!unknown.is_estimate(Grade::Five));
        let known = GradingKey { known: true, ..unknown };
        assert!(!known.is_estimate(Grade::Three));
    }

    #[test]
    fn validates_threshold_order_and_range() {
        assert!(GradingKey::default_unknown().is_valid());
        let unordered = GradingKey { thresholds: [8000, 8000, 6000, 5000], known: true };
        assert!(!unordered.is_valid());
        let too_high = GradingKey { thresholds: [10_000, 7500, 6250, 5000], known: true };
        assert!(!too_high.is_valid());
    }
}
