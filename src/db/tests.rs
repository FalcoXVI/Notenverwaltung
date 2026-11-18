//! Tests of the persistence layer against real (temporary) SQLite files.

use std::path::PathBuf;

use jiff::civil::Date;

use super::migrations::{current_version, stored_version};
use super::{CategoryDraft, Database, DbError, SubjectDraft, create_auto_backup};
use crate::calc::TrafficThresholds;
use crate::domain::{
    CategoryKind, EntryValue, Grade, GradingKey, Points, SemesterData, SemesterId, fraction,
};

fn date(day: i8) -> Date {
    jiff::civil::date(2026, 11, day)
}

fn points_value(achieved: i64, max: i64, actual: Option<Grade>) -> EntryValue {
    EntryValue::Points {
        achieved: Points::from_hundredths(achieved),
        max: Points::from_hundredths(max),
        actual_grade: actual,
    }
}

fn category(name: &str, weight: u32, kind: CategoryKind, best_n: Option<u32>) -> CategoryDraft {
    CategoryDraft { id: None, name: name.into(), weight_percent: weight, kind, best_n }
}

fn math_draft() -> SubjectDraft {
    SubjectDraft {
        id: None,
        name: "Mathematik".into(),
        key: GradingKey { thresholds: [9000, 8000, 6500, 5000], known: true },
        categories: vec![
            category("Schularbeit", 60, CategoryKind::Points, None),
            category("Test", 20, CategoryKind::Points, Some(1)),
            category("Mitarbeit", 20, CategoryKind::Grade, None),
        ],
    }
}

/// A database with one semester, one subject and three entries.
fn sample_db() -> (Database, SemesterId) {
    let mut db = Database::open_in_memory().unwrap();
    let semester = db.insert_semester(3, "WS 2026/27").unwrap();
    db.save_subject(semester, &math_draft()).unwrap();
    let categories: Vec<_> = db.load_all().unwrap()[0].subjects[0]
        .categories
        .iter()
        .map(|c| c.category.id)
        .collect();
    db.insert_entry(categories[0], date(3), "1. SA", &points_value(3650, 4800, Some(Grade::Two)))
        .unwrap();
    db.insert_entry(categories[1], date(10), "", &points_value(1250, 2000, None)).unwrap();
    db.insert_entry(categories[2], date(12), "Referat", &EntryValue::Grade(Grade::One)).unwrap();
    (db, semester)
}

/// A fresh directory below the system temp folder, unique per test.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("notenverwaltung-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn entry_count(data: &[SemesterData]) -> usize {
    data.iter()
        .flat_map(|s| &s.subjects)
        .flat_map(|s| &s.categories)
        .map(|c| c.entries.len())
        .sum()
}

#[test]
fn new_database_is_migrated_to_current_version() {
    let db = Database::open_in_memory().unwrap();
    assert_eq!(stored_version(&db.conn).unwrap(), current_version());
    assert!(db.load_all().unwrap().is_empty());
}

#[test]
fn everything_round_trips() {
    let (db, _) = sample_db();
    let data = db.load_all().unwrap();
    let semester = &data[0];
    assert_eq!(semester.semester.display_name(), "3. Semester (WS 2026/27)");

    let subject = &semester.subjects[0];
    assert_eq!(subject.subject.name, "Mathematik");
    assert_eq!(subject.subject.key, math_draft().key);
    let names: Vec<_> = subject.categories.iter().map(|c| c.category.name.as_str()).collect();
    assert_eq!(names, ["Schularbeit", "Test", "Mitarbeit"]);
    assert_eq!(subject.categories[1].category.best_n, Some(1));
    assert_eq!(subject.categories[1].category.kind, CategoryKind::Points);

    let exam = &subject.categories[0].entries[0];
    assert_eq!(exam.date, date(3));
    assert_eq!(exam.description, "1. SA");
    assert_eq!(exam.value, points_value(3650, 4800, Some(Grade::Two)));
    assert_eq!(subject.categories[2].entries[0].value, EntryValue::Grade(Grade::One));
}

#[test]
fn entries_can_be_updated_and_deleted() {
    let (mut db, _) = sample_db();
    let mut entry = db.load_all().unwrap()[0].subjects[0].categories[2].entries[0].clone();
    entry.value = EntryValue::Grade(Grade::Three);
    entry.description = "geändert".into();
    db.update_entry(&entry).unwrap();
    let loaded = db.load_all().unwrap()[0].subjects[0].categories[2].entries[0].clone();
    assert_eq!(loaded, entry);

    db.delete_entry(entry.id).unwrap();
    assert_eq!(entry_count(&db.load_all().unwrap()), 2);
}

#[test]
fn saving_a_subject_replaces_its_categories() {
    let (mut db, semester) = sample_db();
    let subject = db.load_all().unwrap()[0].subjects[0].clone();

    // Keep "Schularbeit" (renamed, new weight), drop "Test", keep
    // "Mitarbeit", add "Hausübung".
    let existing = |index: usize, name: &str, weight| CategoryDraft {
        id: Some(subject.categories[index].category.id),
        name: name.into(),
        weight_percent: weight,
        kind: subject.categories[index].category.kind,
        best_n: None,
    };
    let draft = SubjectDraft {
        id: Some(subject.subject.id),
        name: "Angewandte Mathematik".into(),
        key: GradingKey::default_unknown(),
        categories: vec![
            existing(0, "SA", 50),
            category("Hausübung", 10, CategoryKind::Grade, None),
            existing(2, "Mitarbeit", 40),
        ],
    };
    db.save_subject(semester, &draft).unwrap();

    let data = db.load_all().unwrap();
    let saved = &data[0].subjects[0];
    assert_eq!(saved.subject.name, "Angewandte Mathematik");
    assert!(!saved.subject.key.known);
    let summary: Vec<_> = saved
        .categories
        .iter()
        .map(|c| (c.category.name.as_str(), c.category.weight_percent, c.entries.len()))
        .collect();
    // Entries of kept categories survive, the entry of "Test" is gone.
    assert_eq!(summary, [("SA", 50, 1), ("Hausübung", 10, 0), ("Mitarbeit", 40, 1)]);
}

#[test]
fn final_grades_are_stored() {
    let (mut db, _) = sample_db();
    let id = db.load_all().unwrap()[0].subjects[0].subject.id;
    db.set_final_grades(id, Some(Grade::Five), Some(Grade::Four)).unwrap();
    let subject = db.load_all().unwrap()[0].subjects[0].subject.clone();
    assert_eq!(subject.report_grade, Some(Grade::Five));
    assert_eq!(subject.exam_grade, Some(Grade::Four));
    assert!(!subject.is_open_negative());
}

#[test]
fn copying_a_semester_takes_structure_but_no_entries_or_final_grades() {
    let (mut db, semester) = sample_db();
    let id = db.load_all().unwrap()[0].subjects[0].subject.id;
    db.set_final_grades(id, Some(Grade::Two), None).unwrap();

    db.copy_semester(semester, 4, "SS 2027").unwrap();
    let data = db.load_all().unwrap();
    assert_eq!(data.len(), 2);
    let (original, copy) = (&data[0].subjects[0], &data[1].subjects[0]);

    assert_eq!(data[1].semester.number, 4);
    assert_eq!(copy.subject.name, original.subject.name);
    assert_eq!(copy.subject.key, original.subject.key);
    assert_eq!(copy.subject.report_grade, None);
    assert_eq!(copy.categories.len(), 3);
    for (a, b) in original.categories.iter().zip(&copy.categories) {
        assert_eq!(a.category.name, b.category.name);
        assert_eq!(a.category.weight_percent, b.category.weight_percent);
        assert_eq!(a.category.kind, b.category.kind);
        assert_eq!(a.category.best_n, b.category.best_n);
        assert_ne!(a.category.id, b.category.id);
        assert!(b.entries.is_empty());
    }
    // The original keeps its entries.
    assert_eq!(entry_count(&data), 3);
}

#[test]
fn deleting_cascades_to_children() {
    let (mut db, semester) = sample_db();
    db.delete_semester(semester).unwrap();
    assert!(db.load_all().unwrap().is_empty());
    let orphans: i64 = db
        .conn
        .query_row(
            "SELECT (SELECT COUNT(*) FROM subject) + (SELECT COUNT(*) FROM category)
                  + (SELECT COUNT(*) FROM entry)",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(orphans, 0);
}

#[test]
fn database_rejects_invalid_entries() {
    let (db, _) = sample_db();
    let result = db.conn.execute(
        "INSERT INTO entry (category_id, date, grade, points_achieved, points_max)
         SELECT id, '2026-11-17', 2, 100, 200 FROM category LIMIT 1",
        [],
    );
    assert!(result.is_err(), "an entry must not be grade and points at once");
    let result = db.conn.execute(
        "INSERT INTO entry (category_id, date, grade) SELECT id, '2026-11-17', 6 FROM category LIMIT 1",
        [],
    );
    assert!(result.is_err(), "grade 6 must be rejected");
}

#[test]
fn traffic_thresholds_default_and_round_trip() {
    let mut db = Database::open_in_memory().unwrap();
    assert_eq!(db.load_traffic_thresholds().unwrap(), TrafficThresholds::default());
    let custom = TrafficThresholds { green_max: fraction(5, 2), yellow_max: fraction(15, 4) };
    db.save_traffic_thresholds(&custom).unwrap();
    assert_eq!(db.load_traffic_thresholds().unwrap(), custom);
}

#[test]
fn data_survives_closing_and_reopening_the_file() {
    let dir = temp_dir("reopen");
    let file = dir.join("noten.db");
    {
        let mut db = Database::open(&file).unwrap();
        db.insert_semester(1, "").unwrap();
    } // `db` is dropped here, which closes the file.
    let db = Database::open(&file).unwrap();
    assert_eq!(db.load_all().unwrap().len(), 1);
    drop(db);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn backup_and_restore_bring_back_the_old_state() {
    let dir = temp_dir("backup");
    let backup_file = dir.join("sicherung.db");
    let (mut db, semester) = sample_db();
    let before = db.load_all().unwrap();
    db.backup_to(&backup_file).unwrap();

    db.delete_semester(semester).unwrap();
    assert!(db.load_all().unwrap().is_empty());

    db.restore_from(&backup_file).unwrap();
    assert_eq!(db.load_all().unwrap(), before);
    // The restored database is fully usable, including foreign keys.
    db.delete_semester(semester).unwrap();
    assert_eq!(entry_count(&db.load_all().unwrap()), 0);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn backup_overwrites_an_existing_file() {
    let dir = temp_dir("overwrite");
    let backup_file = dir.join("sicherung.db");
    let (mut db, semester) = sample_db();
    db.backup_to(&backup_file).unwrap();
    db.delete_semester(semester).unwrap();
    db.backup_to(&backup_file).unwrap();

    let mut other = Database::open_in_memory().unwrap();
    other.restore_from(&backup_file).unwrap();
    assert!(other.load_all().unwrap().is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn restoring_a_wrong_file_is_refused_and_keeps_the_data() {
    let dir = temp_dir("invalid");
    let text_file = dir.join("kein-backup.db");
    std::fs::write(&text_file, "this is not a database").unwrap();
    let empty_db = dir.join("leer.db");
    rusqlite::Connection::open(&empty_db).unwrap().execute_batch("CREATE TABLE x (y)").unwrap();

    let (mut db, _) = sample_db();
    let before = db.load_all().unwrap();
    for file in [&text_file, &empty_db, &dir.join("gibt-es-nicht.db")] {
        let error = db.restore_from(file).unwrap_err();
        assert!(matches!(error, DbError::InvalidBackup(_)), "{file:?}: {error}");
    }
    assert_eq!(db.load_all().unwrap(), before);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn database_of_a_newer_version_is_refused() {
    let dir = temp_dir("newer");
    let file = dir.join("noten.db");
    drop(Database::open(&file).unwrap());
    rusqlite::Connection::open(&file)
        .unwrap()
        .pragma_update(None, "user_version", current_version() + 1)
        .unwrap();
    let result = Database::open(&file);
    assert!(matches!(result, Err(DbError::NewerSchema { .. })));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn opening_with_auto_backup_saves_existing_data_only() {
    let dir = temp_dir("open-backup");
    let file = dir.join("noten.db");
    let backups = dir.join("backups");

    // First start: the file is new, so there is nothing to back up.
    let (mut db, warning) = Database::open_with_auto_backup(&file, &backups).unwrap();
    assert!(warning.is_none());
    assert!(!backups.exists());
    db.insert_semester(1, "").unwrap();
    drop(db);

    // Second start: the existing data is backed up before anything else.
    let (db, warning) = Database::open_with_auto_backup(&file, &backups).unwrap();
    assert!(warning.is_none());
    let backup = std::fs::read_dir(&backups).unwrap().next().unwrap().unwrap().path();
    let mut restored = Database::open_in_memory().unwrap();
    restored.restore_from(&backup).unwrap();
    assert_eq!(restored.load_all().unwrap(), db.load_all().unwrap());
    drop(db);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn auto_backup_keeps_only_the_newest_ten() {
    let dir = temp_dir("auto");
    // Twelve old backups plus one unrelated file that must be left alone.
    for index in 0..12 {
        std::fs::write(dir.join(format!("noten-20200101-0000{index:02}.db")), "old").unwrap();
    }
    std::fs::write(dir.join("eigene-datei.txt"), "keep me").unwrap();

    let (db, _) = sample_db();
    let created = create_auto_backup(&db, &dir).unwrap();
    assert!(created.exists());

    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names.len(), 11, "10 backups + the unrelated file: {names:?}");
    assert!(names.contains(&"eigene-datei.txt".to_string()));
    assert!(!names.contains(&"noten-20200101-000000.db".to_string()));
    assert!(names.contains(&"noten-20200101-000011.db".to_string()));
    std::fs::remove_dir_all(dir).unwrap();
}
