use num_rational::Ratio;

/// Exact rational number used for every grade calculation.
///
/// Floating point (`f64`) is deliberately avoided: an average of exactly 4
/// must never become 4.000000001 and thereby a failing grade.
pub type Fraction = Ratio<i64>;

/// Shorthand for building a fraction `numerator / denominator`.
pub fn fraction(numerator: i64, denominator: i64) -> Fraction {
    Ratio::new(numerator, denominator)
}

/// Formats a non-negative fraction with two decimals and a German comma.
///
/// The value is rounded *up*, matching the "always round up" grading rule:
/// 4.001 is shown as "4,01" instead of a misleading "4,00".
pub fn format_ceil_2(value: Fraction) -> String {
    let hundredths = (value * 100).ceil().to_integer();
    format!("{},{:02}", hundredths / 100, hundredths % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_two_decimals_rounding_up() {
        assert_eq!(format_ceil_2(fraction(4, 1)), "4,00");
        assert_eq!(format_ceil_2(fraction(4001, 1000)), "4,01");
        assert_eq!(format_ceil_2(fraction(7, 3)), "2,34");
        assert_eq!(format_ceil_2(fraction(5, 2)), "2,50");
    }
}
