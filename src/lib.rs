//! Notenverwaltung – manage school grades per semester and subject.
//!
//! Dependencies only point downwards:
//! `ui` → `db` / `export` → `calc` → `domain`.

pub mod calc;
pub mod db;
pub mod domain;
pub mod export;
pub mod ui;
