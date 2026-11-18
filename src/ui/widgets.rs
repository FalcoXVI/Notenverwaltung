//! Small building blocks shared by several views.

use egui::{Color32, RichText, Ui};

use crate::calc::{Graded, Overall, TrafficLight};
use crate::domain::{Fraction, Grade, format_ceil_2, format_hundredths};

/// Red used for errors and negative grades.
pub const RED: Color32 = Color32::from_rgb(0xd1, 0x3b, 0x3b);
const GREEN: Color32 = Color32::from_rgb(0x2e, 0xa0, 0x43);
const YELLOW: Color32 = Color32::from_rgb(0xe0, 0xa8, 0x00);

/// Draws the traffic light as a filled circle; gray if there is no grade.
pub fn traffic_dot(ui: &mut Ui, light: Option<TrafficLight>) {
    let color = match light {
        Some(TrafficLight::Green) => GREEN,
        Some(TrafficLight::Yellow) => YELLOW,
        Some(TrafficLight::Red) => RED,
        None => Color32::GRAY,
    };
    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 6.5, color);
}

/// Drop-down for a grade that must be chosen.
pub fn grade_combo(ui: &mut Ui, id_salt: &str, value: &mut Grade) {
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(grade_with_label(*value))
        .show_ui(ui, |ui| {
            for grade in Grade::ALL {
                ui.selectable_value(value, grade, grade_with_label(grade));
            }
        });
}

/// Drop-down for a grade that may be left empty ("–").
/// Returns `true` if the user changed the value.
pub fn optional_grade_combo(ui: &mut Ui, id_salt: &str, value: &mut Option<Grade>) -> bool {
    let before = *value;
    let selected = value.map_or("–".to_string(), grade_with_label);
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(value, None, "–");
            for grade in Grade::ALL {
                ui.selectable_value(value, Some(grade), grade_with_label(grade));
            }
        });
    *value != before
}

/// "3 – Befriedigend"
pub fn grade_with_label(grade: Grade) -> String {
    format!("{grade} – {}", grade.label())
}

/// The rounded subject grade, e.g. "3" or "~3" if estimated.
pub fn overall_grade_text(overall: &Overall) -> String {
    estimate_marked(overall.grade.to_string(), overall.estimated)
}

/// A category grade, e.g. "2", "2,50" or "~2".
pub fn graded_text(graded: &Graded) -> String {
    let text = if graded.grade.is_integer() {
        graded.grade.to_integer().to_string()
    } else {
        format_ceil_2(graded.grade)
    };
    estimate_marked(text, graded.estimated)
}

/// Prefixes "~" to values that depend on an assumed grading key.
pub fn estimate_marked(text: String, estimated: bool) -> String {
    if estimated { format!("~{text}") } else { text }
}

/// A ratio as percentage with up to two decimals, e.g. "76,04 %".
pub fn percent_text(ratio: Fraction) -> String {
    let hundredths = (ratio * 10_000).floor().to_integer();
    format!("{} %", format_hundredths(hundredths))
}

/// Text in the color of a grade: red for a 5, default otherwise.
pub fn grade_colored(text: String, grade: Grade) -> RichText {
    let text = RichText::new(text).strong();
    if grade.is_positive() { text } else { text.color(RED) }
}

/// A red validation or error message.
pub fn error_label(ui: &mut Ui, message: &str) {
    ui.colored_label(RED, message);
}
