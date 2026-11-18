use std::fmt;

use super::decimal::{format_hundredths, parse_hundredths};

/// A number of points, stored as an integer count of hundredths so that
/// half or quarter points are exact (12.5 points are stored as 1250).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Points(i64);

impl Points {
    /// Builds points from a raw count of hundredths (as stored in the DB).
    pub fn from_hundredths(hundredths: i64) -> Points {
        Points(hundredths)
    }

    /// The raw count of hundredths.
    pub fn hundredths(self) -> i64 {
        self.0
    }

    /// Parses user input such as "12", "12,5" or "12.25"; `None` if the
    /// text is not a non-negative number with at most two decimals.
    pub fn parse(text: &str) -> Option<Points> {
        parse_hundredths(text).map(Points)
    }
}

impl fmt::Display for Points {
    /// Formats with a German comma and without trailing zeros: "12", "12,5".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_hundredths(self.0))
    }
}
