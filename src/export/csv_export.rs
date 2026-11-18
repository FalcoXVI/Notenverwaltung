use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::calc::{Overall, evaluate_subject};
use crate::domain::{Entry, EntryValue, Grade, SemesterData, SubjectData, format_ceil_2};

const HEADER: [&str; 15] = [
    "Semester",
    "Fach",
    "Schnitt",
    "Aktuelle Note",
    "Zeugnisnote",
    "Semesterprüfung",
    "Kategorie",
    "Gewicht %",
    "Datum",
    "Beschreibung",
    "Note",
    "Punkte",
    "Max. Punkte",
    "Tatsächliche Note",
    "Zählt",
];

/// Writes all entries of all semesters into a CSV file that Excel opens
/// directly: semicolon separated, UTF-8 with byte order mark.
pub fn export_csv_file(path: &Path, semesters: &[SemesterData]) -> io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    // The byte order mark tells Excel that the file is UTF-8 (umlauts!).
    file.write_all("\u{feff}".as_bytes())?;
    write_csv(&mut file, semesters)?;
    file.flush()
}

/// Writes the CSV text to any writer (a file, or a byte buffer in tests).
///
/// One line per entry. Subjects and categories without entries get a line
/// with empty entry columns so that they still show up.
pub fn write_csv(writer: impl Write, semesters: &[SemesterData]) -> io::Result<()> {
    let mut csv = csv::WriterBuilder::new().delimiter(b';').from_writer(writer);
    csv.write_record(HEADER)?;
    for semester in semesters {
        let semester_name = semester.semester.display_name();
        for subject in &semester.subjects {
            for line in subject_lines(subject) {
                let mut record = vec![semester_name.clone()];
                record.extend(line);
                csv.write_record(&record)?;
            }
        }
    }
    csv.flush()
}

/// All lines of one subject, without the leading semester column.
fn subject_lines(data: &SubjectData) -> Vec<Vec<String>> {
    let result = evaluate_subject(data);
    let subject_columns = vec![
        data.subject.name.clone(),
        result.overall.map_or(String::new(), |o| format_ceil_2(o.average)),
        result.overall.map_or(String::new(), current_grade_text),
        optional_grade_text(data.subject.report_grade),
        optional_grade_text(data.subject.exam_grade),
    ];
    let empty_columns = |count: usize| vec![String::new(); count];

    if data.categories.is_empty() {
        return vec![[subject_columns, empty_columns(9)].concat()];
    }

    let mut lines = Vec::new();
    for (category, category_result) in data.categories.iter().zip(&result.categories) {
        let category_columns = vec![
            category.category.name.clone(),
            category.category.weight_percent.to_string(),
        ];
        if category.entries.is_empty() {
            lines.push([subject_columns.clone(), category_columns.clone(), empty_columns(7)].concat());
        }
        for (entry, counted) in category.entries.iter().zip(&category_result.counted) {
            lines.push(
                [subject_columns.clone(), category_columns.clone(), entry_columns(entry, *counted)]
                    .concat(),
            );
        }
    }
    lines
}

/// The seven columns describing a single entry.
fn entry_columns(entry: &Entry, counted: bool) -> Vec<String> {
    let (grade, achieved, max, actual) = match entry.value {
        EntryValue::Grade(grade) => (grade.to_string(), String::new(), String::new(), String::new()),
        EntryValue::Points { achieved, max, actual_grade } => (
            String::new(),
            achieved.to_string(),
            max.to_string(),
            optional_grade_text(actual_grade),
        ),
    };
    vec![
        entry.date.strftime("%d.%m.%Y").to_string(),
        entry.description.clone(),
        grade,
        achieved,
        max,
        actual,
        if counted { "ja" } else { "nein" }.to_string(),
    ]
}

/// The rounded grade, prefixed with "~" if it is only an estimate.
fn current_grade_text(overall: Overall) -> String {
    if overall.estimated {
        format!("~{}", overall.grade)
    } else {
        overall.grade.to_string()
    }
}

fn optional_grade_text(grade: Option<Grade>) -> String {
    grade.map_or(String::new(), |g| g.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Category, CategoryData, CategoryId, CategoryKind, EntryId, GradingKey, Points, Semester,
        SemesterId, Subject, SubjectId,
    };

    fn entry(day: i8, description: &str, value: EntryValue) -> Entry {
        Entry {
            id: EntryId(0),
            category_id: CategoryId(0),
            date: jiff::civil::date(2026, 11, day),
            description: description.into(),
            value,
        }
    }

    fn category(name: &str, kind: CategoryKind, best_n: Option<u32>, entries: Vec<Entry>) -> CategoryData {
        CategoryData {
            category: Category {
                id: CategoryId(0),
                subject_id: SubjectId(0),
                name: name.into(),
                weight_percent: 50,
                kind,
                best_n,
            },
            entries,
        }
    }

    fn points(achieved: i64, max: i64) -> EntryValue {
        EntryValue::Points {
            achieved: Points::from_hundredths(achieved),
            max: Points::from_hundredths(max),
            actual_grade: None,
        }
    }

    #[test]
    fn writes_one_line_per_entry_with_subject_summary() {
        let semesters = vec![SemesterData {
            semester: Semester { id: SemesterId(1), number: 3, label: "WS".into() },
            subjects: vec![SubjectData {
                subject: Subject {
                    id: SubjectId(1),
                    semester_id: SemesterId(1),
                    name: "Deutsch".into(),
                    key: GradingKey::default_unknown(),
                    report_grade: Some(Grade::Three),
                    exam_grade: None,
                },
                categories: vec![
                    category("Test", CategoryKind::Points, Some(1), vec![
                        entry(3, "Test; 1", points(1050, 2000)),
                        entry(10, "", points(1650, 2000)),
                    ]),
                    category("Mitarbeit", CategoryKind::Grade, None, vec![]),
                ],
            }],
        }];

        let mut buffer = Vec::new();
        write_csv(&mut buffer, &semesters).unwrap();
        let text = String::from_utf8(buffer).unwrap();
        let lines: Vec<&str> = text.lines().collect();

        assert_eq!(lines.len(), 4);
        assert!(lines[0].starts_with("Semester;Fach;Schnitt;"));
        // 16.5/20 = 82.5 % -> an (estimated) 2; the worse test is dropped.
        // The description contains a semicolon and must be quoted.
        assert_eq!(
            lines[1],
            "3. Semester (WS);Deutsch;2,00;~2;3;;Test;50;03.11.2026;\"Test; 1\";;10,5;20;;nein"
        );
        assert_eq!(
            lines[2],
            "3. Semester (WS);Deutsch;2,00;~2;3;;Test;50;10.11.2026;;;16,5;20;;ja"
        );
        assert_eq!(lines[3], "3. Semester (WS);Deutsch;2,00;~2;3;;Mitarbeit;50;;;;;;;");
    }
}
