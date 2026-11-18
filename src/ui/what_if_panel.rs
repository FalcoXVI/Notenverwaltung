use super::widgets::{error_label, estimate_marked};
use crate::calc::{Requirement, WhatIf, required_for_target};
use crate::domain::{CategoryData, CategoryId, CategoryKind, EntryValue, Grade, Points, SubjectData};

/// Larger maxima are refused so that trying every half point stays fast.
const MAX_POINTS_LIMIT: i64 = 2000;

/// Input state of the what-if panel.
#[derive(Default)]
pub struct WhatIfState {
    /// Index of the category the next performance belongs to.
    category_index: usize,
    /// Maximum points of the next performance, as typed.
    max_text: String,
    /// Category for which `max_text` was last pre-filled.
    prefilled_for: Option<CategoryId>,
}

/// Draws the panel "what do I need in the next performance?".
pub fn show(ui: &mut egui::Ui, data: &SubjectData, state: &mut WhatIfState) {
    ui.heading("Was-wäre-wenn");
    ui.label("Was brauche ich bei der nächsten Leistung?");
    ui.add_space(6.0);

    if data.categories.is_empty() {
        ui.label("Das Fach hat noch keine Kategorien.");
        return;
    }
    state.category_index = state.category_index.min(data.categories.len() - 1);

    egui::ComboBox::from_id_salt("what_if_category")
        .width(200.0)
        .selected_text(&data.categories[state.category_index].category.name)
        .show_ui(ui, |ui| {
            for (index, category) in data.categories.iter().enumerate() {
                ui.selectable_value(&mut state.category_index, index, &category.category.name);
            }
        });
    let category = &data.categories[state.category_index];
    prefill_max(state, category);

    let next_max = match category.category.kind {
        CategoryKind::Grade => Some(Points::from_hundredths(0)),
        CategoryKind::Points => {
            ui.horizontal(|ui| {
                ui.label("Maximale Punkte:");
                ui.add(egui::TextEdit::singleline(&mut state.max_text).desired_width(60.0));
            });
            parse_max(ui, &state.max_text)
        }
    };
    ui.add_space(8.0);

    if let Some(next_max) = next_max {
        show_answers(ui, data, state.category_index, next_max);
    }
    if category.category.best_n.is_some() {
        ui.add_space(6.0);
        ui.label(egui::RichText::new("„Nur die besten zählen“ ist berücksichtigt.").weak());
    }
}

/// Suggests the maximum of the latest entry whenever the category changes.
fn prefill_max(state: &mut WhatIfState, category: &CategoryData) {
    if state.prefilled_for == Some(category.category.id) {
        return;
    }
    state.prefilled_for = Some(category.category.id);
    state.max_text = match category.entries.last().map(|entry| entry.value) {
        Some(EntryValue::Points { max, .. }) => max.to_string(),
        _ => String::new(),
    };
}

/// Parses the maximum points; shows a hint and returns `None` if the
/// input cannot be used.
fn parse_max(ui: &mut egui::Ui, text: &str) -> Option<Points> {
    if text.trim().is_empty() {
        ui.label(egui::RichText::new("Bitte die maximale Punktezahl eingeben.").weak());
        return None;
    }
    match Points::parse(text) {
        Some(max) if max.hundredths() > 0 && max.hundredths() <= MAX_POINTS_LIMIT * 100 => Some(max),
        _ => {
            error_label(ui, &format!("Bitte eine Zahl zwischen 0 und {MAX_POINTS_LIMIT} eingeben."));
            None
        }
    }
}

/// One line per target grade 1–4.
fn show_answers(ui: &mut egui::Ui, data: &SubjectData, category_index: usize, next_max: Points) {
    let targets = [Grade::One, Grade::Two, Grade::Three, Grade::Four];
    egui::Grid::new("what_if_answers").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
        for target in targets {
            let label = if target == Grade::Four { "Positiv (4):".to_string() } else { format!("Für {target}:") };
            ui.label(label);
            let answer = required_for_target(data, category_index, next_max, target);
            ui.label(answer_text(&answer, next_max));
            ui.end_row();
        }
    });
}

/// German sentence for an answer.
fn answer_text(answer: &WhatIf, next_max: Points) -> String {
    let text = match answer.requirement {
        Requirement::AlreadySafe => "schon sicher".to_string(),
        Requirement::MinPoints(points) => format!("mindestens {points} von {next_max} Punkten"),
        Requirement::WorstGrade(grade) => format!("Note {grade} oder besser"),
        Requirement::Unreachable => "nicht mehr erreichbar".to_string(),
    };
    estimate_marked(text, answer.estimated)
}
