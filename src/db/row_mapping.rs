//! Conversions between database columns and domain types.

use jiff::civil::Date;
use rusqlite::types::Type;

use crate::domain::{CategoryKind, EntryValue, Grade, Points};

/// Date format used in the `entry.date` column.
pub const DATE_FORMAT: &str = "%Y-%m-%d";

/// Converts a stored number into a grade. The CHECK constraints make an
/// invalid value impossible, but a manually edited file is reported as an
/// error instead of crashing the program.
pub fn grade_from_db(number: i64) -> rusqlite::Result<Grade> {
    Grade::from_number(number).ok_or(rusqlite::Error::IntegralValueOutOfRange(0, number))
}

/// Converts an optional stored number into an optional grade.
pub fn optional_grade_from_db(number: Option<i64>) -> rusqlite::Result<Option<Grade>> {
    number.map(grade_from_db).transpose()
}

/// Converts an optional grade into the value written to the database.
pub fn optional_grade_to_db(grade: Option<Grade>) -> Option<i64> {
    grade.map(Grade::number)
}

/// Text stored in `category.kind`.
pub fn kind_to_db(kind: CategoryKind) -> &'static str {
    match kind {
        CategoryKind::Grade => "grade",
        CategoryKind::Points => "points",
    }
}

/// Parses the text stored in `category.kind`.
pub fn kind_from_db(text: &str) -> rusqlite::Result<CategoryKind> {
    match text {
        "grade" => Ok(CategoryKind::Grade),
        "points" => Ok(CategoryKind::Points),
        other => Err(conversion_error(format!("unknown category kind '{other}'"))),
    }
}

/// Parses the text stored in `entry.date`.
pub fn date_from_db(text: &str) -> rusqlite::Result<Date> {
    Date::strptime(DATE_FORMAT, text)
        .map_err(|error| conversion_error(format!("invalid date '{text}': {error}")))
}

/// The four value columns of an entry as written to the database:
/// `(grade, points_achieved, points_max, actual_grade)`.
pub type ValueColumns = (Option<i64>, Option<i64>, Option<i64>, Option<i64>);

/// Splits an entry value into its four columns.
pub fn value_to_db(value: &EntryValue) -> ValueColumns {
    match value {
        EntryValue::Grade(grade) => (Some(grade.number()), None, None, None),
        EntryValue::Points { achieved, max, actual_grade } => (
            None,
            Some(achieved.hundredths()),
            Some(max.hundredths()),
            optional_grade_to_db(*actual_grade),
        ),
    }
}

/// Builds an entry value from its four columns.
pub fn value_from_db(columns: ValueColumns) -> rusqlite::Result<EntryValue> {
    match columns {
        (Some(grade), None, None, None) => Ok(EntryValue::Grade(grade_from_db(grade)?)),
        (None, Some(achieved), Some(max), actual) => Ok(EntryValue::Points {
            achieved: Points::from_hundredths(achieved),
            max: Points::from_hundredths(max),
            actual_grade: optional_grade_from_db(actual)?,
        }),
        _ => Err(conversion_error("entry is neither a grade nor points".into())),
    }
}

/// Wraps a message into the error type rusqlite expects from row mappers.
fn conversion_error(message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, Type::Text, message.into())
}
