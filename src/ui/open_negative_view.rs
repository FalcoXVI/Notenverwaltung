use super::action::Action;
use super::screen::Screen;
use super::store::Store;
use super::widgets::grade_colored;
use crate::domain::{Grade, SemesterData, SubjectData};

/// All subjects with a negative report grade that has not been repaired
/// by a positive semester exam, across all semesters.
pub fn open_negative_subjects(store: &Store) -> Vec<(&SemesterData, &SubjectData)> {
    store
        .semesters()
        .iter()
        .flat_map(|semester| semester.subjects.iter().map(move |subject| (semester, subject)))
        .filter(|(_, subject)| subject.subject.is_open_negative())
        .collect()
}

/// Draws the page listing the open negative subjects.
pub fn show(ui: &mut egui::Ui, store: &Store, actions: &mut Vec<Action>) {
    ui.heading("Offene negative Fächer");
    ui.label(
        "Fächer mit Zeugnisnote 5, die noch nicht durch eine positive Semesterprüfung ausgebessert sind.",
    );
    ui.add_space(8.0);

    let open = open_negative_subjects(store);
    if open.is_empty() {
        ui.label(egui::RichText::new("Keine offenen negativen Fächer.").strong());
        return;
    }

    egui::Grid::new("open_negative_table")
        .num_columns(4)
        .striped(true)
        .spacing([24.0, 8.0])
        .show(ui, |ui| {
            for heading in ["Semester", "Fach", "Zeugnis", "Semesterprüfung"] {
                ui.label(egui::RichText::new(heading).weak());
            }
            ui.end_row();
            for (semester, subject) in open {
                ui.label(semester.semester.display_name());
                if ui.link(egui::RichText::new(&subject.subject.name).strong()).clicked() {
                    actions.push(Action::Navigate(Screen::Subject(subject.subject.id)));
                }
                ui.label(grade_colored("5".to_string(), Grade::Five));
                match subject.subject.exam_grade {
                    Some(grade) => ui.label(grade_colored(format!("{grade} (nicht bestanden)"), grade)),
                    None => ui.label("noch nicht abgelegt"),
                };
                ui.end_row();
            }
        });
}
