//! Exact decimal numbers with two decimals, kept as an integer count of
//! hundredths ("12,5" <-> 1250). Used for points, percentages and limits.

/// Parses user input such as "12", "12,5" or "12.25" into hundredths.
///
/// Returns `None` for negative numbers, more than two decimals or anything
/// that is not a number.
pub fn parse_hundredths(text: &str) -> Option<i64> {
    let text = text.trim().replace(',', ".");
    let (whole, decimals) = match text.split_once('.') {
        Some((whole, decimals)) => (whole, decimals),
        None => (text.as_str(), ""),
    };
    let all_digits = |s: &str| s.chars().all(|c| c.is_ascii_digit());
    if whole.is_empty() || whole.len() > 9 || decimals.len() > 2 {
        return None;
    }
    if !all_digits(whole) || !all_digits(decimals) {
        return None;
    }
    // Pad "5" to "50" so that "12,5" means 50 hundredths, not 5.
    let padded = format!("{decimals:0<2}");
    let whole: i64 = whole.parse().ok()?;
    let decimals: i64 = padded.parse().ok()?;
    Some(whole * 100 + decimals)
}

/// Formats hundredths with a German comma and without trailing zeros:
/// 1200 -> "12", 1250 -> "12,5", 705 -> "7,05".
pub fn format_hundredths(hundredths: i64) -> String {
    let (whole, rest) = (hundredths / 100, hundredths % 100);
    if rest == 0 {
        format!("{whole}")
    } else if rest % 10 == 0 {
        format!("{whole},{}", rest / 10)
    } else {
        format!("{whole},{rest:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_german_and_english_decimals() {
        assert_eq!(parse_hundredths("12"), Some(1200));
        assert_eq!(parse_hundredths("12,5"), Some(1250));
        assert_eq!(parse_hundredths(" 0.25 "), Some(25));
        assert_eq!(parse_hundredths("7,05"), Some(705));
    }

    #[test]
    fn rejects_invalid_input() {
        for text in ["", "-1", "1,234", "abc", "1,2,3", ",5"] {
            assert_eq!(parse_hundredths(text), None, "input: {text:?}");
        }
    }

    #[test]
    fn formatting_round_trips() {
        for text in ["12", "12,5", "7,05", "0"] {
            assert_eq!(format_hundredths(parse_hundredths(text).unwrap()), text);
        }
    }
}
