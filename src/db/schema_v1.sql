-- Schema version 1. Never edit this file after a release; add a new
-- migration file instead so existing databases can be upgraded.

CREATE TABLE semester (
    id     INTEGER PRIMARY KEY,
    number INTEGER NOT NULL CHECK (number > 0),
    label  TEXT    NOT NULL DEFAULT ''
);

CREATE TABLE subject (
    id           INTEGER PRIMARY KEY,
    semester_id  INTEGER NOT NULL REFERENCES semester (id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,
    -- 0 = thresholds are only the assumed default key
    key_known    INTEGER NOT NULL DEFAULT 0 CHECK (key_known IN (0, 1)),
    -- thresholds in hundredths of a percent (87.5 % = 8750)
    threshold_1  INTEGER NOT NULL,
    threshold_2  INTEGER NOT NULL,
    threshold_3  INTEGER NOT NULL,
    threshold_4  INTEGER NOT NULL,
    report_grade INTEGER CHECK (report_grade BETWEEN 1 AND 5),
    exam_grade   INTEGER CHECK (exam_grade BETWEEN 1 AND 5)
);
CREATE INDEX subject_semester ON subject (semester_id);

CREATE TABLE category (
    id             INTEGER PRIMARY KEY,
    subject_id     INTEGER NOT NULL REFERENCES subject (id) ON DELETE CASCADE,
    name           TEXT    NOT NULL,
    weight_percent INTEGER NOT NULL CHECK (weight_percent BETWEEN 0 AND 100),
    kind           TEXT    NOT NULL CHECK (kind IN ('grade', 'points')),
    best_n         INTEGER CHECK (best_n > 0),
    sort_order     INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX category_subject ON category (subject_id);

CREATE TABLE entry (
    id              INTEGER PRIMARY KEY,
    category_id     INTEGER NOT NULL REFERENCES category (id) ON DELETE CASCADE,
    date            TEXT    NOT NULL,           -- ISO date, e.g. 2026-11-17
    description     TEXT    NOT NULL DEFAULT '',
    grade           INTEGER CHECK (grade BETWEEN 1 AND 5),
    -- points in hundredths (12.5 points = 1250)
    points_achieved INTEGER CHECK (points_achieved >= 0),
    points_max      INTEGER CHECK (points_max > 0),
    actual_grade    INTEGER CHECK (actual_grade BETWEEN 1 AND 5),
    -- an entry is either a grade or points, never both and never neither
    CHECK (
        (grade IS NOT NULL AND points_achieved IS NULL AND points_max IS NULL AND actual_grade IS NULL)
        OR (grade IS NULL AND points_achieved IS NOT NULL AND points_max IS NOT NULL)
    )
);
CREATE INDEX entry_category ON entry (category_id);

CREATE TABLE setting (
    key   TEXT PRIMARY KEY,
    value INTEGER NOT NULL
);
