use super::action::Action;
use super::command::Command;
use super::dialog::Dialog;
use super::entry_dialog::EntryDialog;
use super::screen::Screen;
use super::store::Store;
use super::subject_dialog::SubjectDialog;
use super::what_if_panel::{self, WhatIfState};
use super::widgets::{
    estimate_marked, grade_colored, graded_text, optional_grade_combo, overall_grade_text,
    percent_text, traffic_dot,
};
use crate::calc::{CategoryResult, SubjectResult, TrafficLight, entry_grade, evaluate_subject};
use crate::domain::{CategoryData, Entry, EntryValue, SubjectData, SubjectId, format_ceil_2};

/// Draws the page of one subject: summary, categories with their entries
/// and, on the right, the what-if panel.
pub fn show(
    ui: &mut egui::Ui,
    store: &Store,
    subject_id: SubjectId,
    what_if: &mut WhatIfState,
    actions: &mut Vec<Action>,
) {
    let Some((semester, data)) = store.subject(subject_id) else {
        return;
    };
    let result = evaluate_subject(data);

    ui.horizontal(|ui| {
        if ui.button("← Zurück").clicked() {
            actions.push(Action::Navigate(Screen::Overview));
        }
        ui.heading(&data.subject.name);
        ui.label(egui::RichText::new(semester.semester.display_name()).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Fach löschen").clicked() {
                actions.push(Action::Confirm {
                    message: format!("„{}“ wird mit allen Einträgen gelöscht.", data.subject.name),
                    command: Command::DeleteSubject(subject_id),
                });
            }
            if ui.button("Fach bearbeiten …").clicked() {
                actions.push(Action::OpenDialog(Dialog::Subject(SubjectDialog::edit(data))));
            }
        });
    });
    ui.add_space(4.0);
    show_summary(ui, store, data, &result);
    show_final_grades(ui, data, actions);
    ui.separator();

    egui::Panel::right("what_if_panel")
        .resizable(false)
        .default_size(330.0)
        .show(ui, |ui| what_if_panel::show(ui, data, what_if));
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (category, category_result) in data.categories.iter().zip(&result.categories) {
                show_category(ui, data, category, category_result, actions);
                ui.add_space(14.0);
            }
        });
    });
}

/// Traffic light, average and current grade in one line.
fn show_summary(ui: &mut egui::Ui, store: &Store, data: &SubjectData, result: &SubjectResult) {
    ui.horizontal(|ui| {
        let light = result.overall.map(|o| TrafficLight::for_average(o.average, store.thresholds()));
        traffic_dot(ui, light);
        match &result.overall {
            Some(overall) => {
                ui.label(format!("Schnitt {}", format_ceil_2(overall.average)));
                ui.label("·");
                ui.label("Aktuelle Note:");
                let text = format!("{} ({})", overall_grade_text(overall), overall.grade.label());
                ui.label(grade_colored(text, overall.grade).size(16.0));
            }
            None => {
                ui.label("Noch keine Einträge – noch keine Note.");
            }
        }
    });
    if !data.subject.key.known {
        ui.label(
            egui::RichText::new(
                "Notenschlüssel noch nicht bekannt: mit ~ markierte Noten sind geschätzt. \
                 Den echten Schlüssel trägst du unter „Fach bearbeiten“ ein.",
            )
            .weak(),
        );
    }
}

/// Drop-downs for the report grade and the semester exam.
fn show_final_grades(ui: &mut egui::Ui, data: &SubjectData, actions: &mut Vec<Action>) {
    let mut report_grade = data.subject.report_grade;
    let mut exam_grade = data.subject.exam_grade;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Zeugnisnote:");
        changed |= optional_grade_combo(ui, "report_grade", &mut report_grade);
        ui.add_space(16.0);
        ui.label("Semesterprüfung:");
        changed |= optional_grade_combo(ui, "exam_grade", &mut exam_grade);
        if data.subject.is_open_negative() {
            ui.label(grade_colored("negativ – noch offen".to_string(), crate::domain::Grade::Five));
        }
    });
    if changed {
        let subject_id = data.subject.id;
        actions.push(Action::Run(Command::SetFinalGrades { subject_id, report_grade, exam_grade }));
    }
}

/// Heading, entry table and "add" button of one category.
fn show_category(
    ui: &mut egui::Ui,
    data: &SubjectData,
    category: &CategoryData,
    result: &CategoryResult,
    actions: &mut Vec<Action>,
) {
    let settings = &category.category;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&settings.name).strong().size(16.0));
        let mut details = format!("{} % · {}", settings.weight_percent, settings.kind.label());
        if let Some(n) = settings.best_n {
            details += &format!(" · nur die besten {n} zählen");
        }
        ui.label(egui::RichText::new(details).weak());
        if let Some(graded) = &result.graded {
            ui.label("→ Note");
            ui.label(egui::RichText::new(graded_text(graded)).strong());
        }
    });

    if !category.entries.is_empty() {
        egui::Grid::new(("entries", settings.id.0))
            .num_columns(5)
            .striped(true)
            .spacing([18.0, 6.0])
            .show(ui, |ui| {
                for (entry, counted) in category.entries.iter().zip(&result.counted) {
                    show_entry_row(ui, data, category, entry, *counted, actions);
                    ui.end_row();
                }
            });
    }
    if ui.button("+ Eintrag").clicked() {
        // Suggest the maximum of the latest entry: tests usually repeat it.
        let suggested_max = match category.entries.last().map(|entry| entry.value) {
            Some(EntryValue::Points { max, .. }) => Some(max),
            _ => None,
        };
        let dialog = EntryDialog::create(settings, suggested_max);
        actions.push(Action::OpenDialog(Dialog::Entry(dialog)));
    }
}

fn show_entry_row(
    ui: &mut egui::Ui,
    data: &SubjectData,
    category: &CategoryData,
    entry: &Entry,
    counted: bool,
    actions: &mut Vec<Action>,
) {
    // Entries that do not count stay visible but are struck through.
    let styled = |text: String| {
        let text = egui::RichText::new(text);
        if counted { text } else { text.strikethrough().weak() }
    };
    let grade = entry_grade(&entry.value, &data.subject.key);

    ui.label(styled(entry.date.strftime("%d.%m.%Y").to_string()));
    ui.label(styled(entry.description.clone()));
    ui.label(styled(result_text(&entry.value)));
    ui.label(styled(estimate_marked(grade.grade.to_string(), grade.estimated)).strong());
    ui.horizontal(|ui| {
        if ui.small_button("Bearbeiten").clicked() {
            let dialog = EntryDialog::edit(&category.category, entry);
            actions.push(Action::OpenDialog(Dialog::Entry(dialog)));
        }
        if ui.small_button("Löschen").clicked() {
            actions.push(Action::Confirm {
                message: format!(
                    "Der Eintrag vom {} in „{}“ wird gelöscht.",
                    entry.date.strftime("%d.%m.%Y"),
                    category.category.name
                ),
                command: Command::DeleteEntry(entry.id),
            });
        }
        if !counted {
            ui.label(egui::RichText::new("zählt nicht").weak());
        }
    });
}

/// What was achieved: "36,5 / 48 (76,04 %)" for points, empty for grades
/// (their grade is shown in the next column anyway).
fn result_text(value: &EntryValue) -> String {
    match value {
        EntryValue::Grade(_) => String::new(),
        EntryValue::Points { achieved, max, .. } => match value.ratio() {
            Some(ratio) => format!("{achieved} / {max} ({})", percent_text(ratio)),
            None => format!("{achieved} / {max}"),
        },
    }
}
