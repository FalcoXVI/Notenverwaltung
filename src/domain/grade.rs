use std::fmt;

/// An Austrian school grade. Only the five valid values can exist, so no
/// other part of the program ever has to check for "grade 0" or "grade 7".
///
/// The derived ordering follows the numbers: `One < Five`, i.e. a *smaller*
/// grade is the *better* one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Grade {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
}

impl Grade {
    /// All grades from best to worst.
    pub const ALL: [Grade; 5] = [
        Grade::One,
        Grade::Two,
        Grade::Three,
        Grade::Four,
        Grade::Five,
    ];

    /// Converts a number into a grade; returns `None` outside 1..=5.
    pub fn from_number(number: i64) -> Option<Grade> {
        match number {
            1 => Some(Grade::One),
            2 => Some(Grade::Two),
            3 => Some(Grade::Three),
            4 => Some(Grade::Four),
            5 => Some(Grade::Five),
            _ => None,
        }
    }

    /// The grade as a number 1..=5.
    pub fn number(self) -> i64 {
        self as i64
    }

    /// Everything except a 5 is a passing grade.
    pub fn is_positive(self) -> bool {
        self != Grade::Five
    }

    /// German name as printed in a report card.
    pub fn label(self) -> &'static str {
        match self {
            Grade::One => "Sehr gut",
            Grade::Two => "Gut",
            Grade::Three => "Befriedigend",
            Grade::Four => "Genügend",
            Grade::Five => "Nicht genügend",
        }
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.number())
    }
}
