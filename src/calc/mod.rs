//! Grade calculation. Pure functions on `domain` types: no GUI, no
//! database, no floating point. Everything here is covered by unit tests.

mod best_n;
mod category;
mod grading;
mod subject;
#[cfg(test)]
mod test_support;
mod traffic_light;
mod what_if;

pub use category::{CategoryResult, Graded, evaluate_category};
pub use grading::{EntryGrade, entry_grade};
pub use subject::{Overall, SubjectResult, evaluate_subject, round_up};
pub use traffic_light::{TrafficLight, TrafficThresholds};
pub use what_if::{Requirement, WhatIf, required_for_target};
