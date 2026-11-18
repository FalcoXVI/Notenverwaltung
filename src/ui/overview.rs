use super::action::Action;
use super::command::Command;
use super::dialog::Dialog;
use super::screen::Screen;
use super::semester_dialog::SemesterDialog;
use super::store::Store;
use super::subject_dialog::SubjectDialog;
use super::widgets::{grade_colored, overall_grade_text, traffic_dot};
use crate::calc::{TrafficLight, evaluate_subject};
use crate::domain::{SemesterData, SemesterId, SubjectData, format_ceil_2};

/// Draws the start page: semester selection and the table of subjects.
pub fn show(ui: &mut egui::Ui, store: &Store, selected: Option<SemesterId>, actions: &mut Vec<Action>) {
    let Some(semester) = selected.and_then(|id| store.semester(id)) else {
        show_welcome(ui, store, actions);
        return;
    };

    show_semester_bar(ui, store, semester, actions);
    ui.separator();
    ui.add_space(4.0);

    if semester.subjects.is_empty() {
        ui.label("In diesem Semester gibt es noch keine Fächer.");
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| show_subject_table(ui, store, semester, actions));
    }
    ui.add_space(8.0);
    if ui.button("+ Neues Fach …").clicked() {
        let dialog = SubjectDialog::create(store, semester.semester.id);
        actions.push(Action::OpenDialog(Dialog::Subject(dialog)));
    }
}

/// Shown while no semester exists yet.
fn show_welcome(ui: &mut egui::Ui, store: &Store, actions: &mut Vec<Action>) {
    ui.heading("Willkommen!");
    ui.label("Lege zuerst ein Semester an, danach die Fächer mit ihren Kategorien.");
    ui.add_space(8.0);
    if ui.button("+ Neues Semester …").clicked() {
        let dialog = SemesterDialog::create(store, None);
        actions.push(Action::OpenDialog(Dialog::Semester(dialog)));
    }
}

/// Semester drop-down with the buttons to create, edit and delete.
fn show_semester_bar(ui: &mut egui::Ui, store: &Store, current: &SemesterData, actions: &mut Vec<Action>) {
    ui.horizontal(|ui| {
        let mut selected = current.semester.id;
        egui::ComboBox::from_id_salt("semester_select")
            .width(240.0)
            .selected_text(current.semester.display_name())
            .show_ui(ui, |ui| {
                for semester in store.semesters() {
                    let name = semester.semester.display_name();
                    ui.selectable_value(&mut selected, semester.semester.id, name);
                }
            });
        if selected != current.semester.id {
            actions.push(Action::SelectSemester(selected));
        }

        if ui.button("+ Neues Semester …").clicked() {
            let dialog = SemesterDialog::create(store, Some(current.semester.id));
            actions.push(Action::OpenDialog(Dialog::Semester(dialog)));
        }
        if ui.button("Bearbeiten …").clicked() {
            let dialog = SemesterDialog::edit(&current.semester);
            actions.push(Action::OpenDialog(Dialog::Semester(dialog)));
        }
        if ui.button("Löschen").clicked() {
            actions.push(Action::Confirm {
                message: format!(
                    "„{}“ wird mit allen {} Fächern und deren Einträgen gelöscht.",
                    current.semester.display_name(),
                    current.subjects.len()
                ),
                command: Command::DeleteSemester(current.semester.id),
            });
        }
    });
}

fn show_subject_table(ui: &mut egui::Ui, store: &Store, semester: &SemesterData, actions: &mut Vec<Action>) {
    egui::Grid::new("subject_table")
        .num_columns(6)
        .striped(true)
        .spacing([24.0, 8.0])
        .show(ui, |ui| {
            for heading in ["", "Fach", "Schnitt", "Aktuelle Note", "Zeugnis", ""] {
                ui.label(egui::RichText::new(heading).weak());
            }
            ui.end_row();
            for subject in &semester.subjects {
                show_subject_row(ui, store, subject, actions);
                ui.end_row();
            }
        });
    ui.add_space(4.0);
    ui.label(egui::RichText::new("~ = geschätzt, weil der Notenschlüssel des Fachs noch nicht bekannt ist").weak());
}

fn show_subject_row(ui: &mut egui::Ui, store: &Store, data: &SubjectData, actions: &mut Vec<Action>) {
    let overall = evaluate_subject(data).overall;
    let open = Action::Navigate(Screen::Subject(data.subject.id));

    traffic_dot(ui, overall.map(|o| TrafficLight::for_average(o.average, store.thresholds())));
    if ui.link(egui::RichText::new(&data.subject.name).strong()).clicked() {
        actions.push(open);
    }
    match overall {
        Some(overall) => {
            ui.label(format_ceil_2(overall.average));
            ui.label(grade_colored(overall_grade_text(&overall), overall.grade));
        }
        None => {
            ui.label("–");
            ui.label("–");
        }
    }
    ui.label(final_grade_text(data));
    ui.label("");
}

/// Report grade, with the exam result if one was taken: "5 → 4".
fn final_grade_text(data: &SubjectData) -> egui::RichText {
    let subject = &data.subject;
    match (subject.report_grade, subject.exam_grade) {
        (Some(report), Some(exam)) => grade_colored(format!("{report} → {exam}"), exam),
        (Some(report), None) => grade_colored(report.to_string(), report),
        (None, _) => egui::RichText::new("–"),
    }
}
