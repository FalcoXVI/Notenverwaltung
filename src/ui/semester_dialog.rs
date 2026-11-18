use super::action::Action;
use super::command::Command;
use super::store::Store;
use crate::domain::{Semester, SemesterId};

/// Dialog for creating a semester (empty or as a copy) or renaming one.
pub struct SemesterDialog {
    /// `None` while creating a new semester.
    id: Option<SemesterId>,
    number: u32,
    label: String,
    /// Semester whose structure is copied; only used for new semesters.
    copy_from: Option<SemesterId>,
    /// Semesters offered as copy source, with their display names.
    sources: Vec<(SemesterId, String)>,
}

impl SemesterDialog {
    /// Dialog for a new semester. Suggests the next number and, if there
    /// is one, copying the semester currently shown.
    pub fn create(store: &Store, current: Option<SemesterId>) -> SemesterDialog {
        let highest = store.semesters().iter().map(|s| s.semester.number).max();
        SemesterDialog {
            id: None,
            number: highest.map_or(1, |n| n + 1),
            label: String::new(),
            copy_from: current,
            sources: store
                .semesters()
                .iter()
                .map(|s| (s.semester.id, s.semester.display_name()))
                .collect(),
        }
    }

    /// Dialog for changing number and label of an existing semester.
    pub fn edit(semester: &Semester) -> SemesterDialog {
        SemesterDialog {
            id: Some(semester.id),
            number: semester.number,
            label: semester.label.clone(),
            copy_from: None,
            sources: Vec::new(),
        }
    }

    /// Draws the dialog.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        egui::Modal::new(egui::Id::unique("semester_dialog")).show(ctx, |ui| {
            ui.set_width(400.0);
            ui.heading(if self.id.is_some() { "Semester bearbeiten" } else { "Neues Semester" });
            ui.add_space(8.0);

            egui::Grid::new("semester_fields").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                ui.label("Nummer:");
                ui.add(egui::DragValue::new(&mut self.number).range(1..=99).suffix(". Semester"));
                ui.end_row();
                ui.label("Bezeichnung:");
                ui.add(egui::TextEdit::singleline(&mut self.label).hint_text("optional, z.B. WS 2026/27"));
                ui.end_row();
            });

            if self.id.is_none() && !self.sources.is_empty() {
                ui.add_space(8.0);
                self.show_copy_choice(ui);
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Speichern").clicked() {
                    actions.push(Action::Run(self.command()));
                }
                if ui.button("Abbrechen").clicked() {
                    actions.push(Action::CloseDialog);
                }
            });
        });
    }

    /// Radio buttons "empty" / "copy from …".
    fn show_copy_choice(&mut self, ui: &mut egui::Ui) {
        if ui.radio(self.copy_from.is_none(), "Leer anlegen").clicked() {
            self.copy_from = None;
        }
        ui.horizontal(|ui| {
            let first_source = self.sources[0].0;
            if ui.radio(self.copy_from.is_some(), "Kopieren von:").clicked() && self.copy_from.is_none() {
                self.copy_from = Some(first_source);
            }
            let selected_name = self
                .sources
                .iter()
                .find(|(id, _)| Some(*id) == self.copy_from)
                .map_or("", |(_, name)| name.as_str());
            egui::ComboBox::from_id_salt("copy_source")
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    for (id, name) in &self.sources {
                        ui.selectable_value(&mut self.copy_from, Some(*id), name);
                    }
                });
        });
        ui.label(
            egui::RichText::new(
                "Kopiert werden Fächer, Kategorien, Gewichtungen und Notenschlüssel – keine Einträge.",
            )
            .weak(),
        );
    }

    /// The database change for the current input.
    fn command(&self) -> Command {
        let label = self.label.trim().to_string();
        match self.id {
            Some(id) => Command::UpdateSemester(Semester { id, number: self.number, label }),
            None => Command::CreateSemester { number: self.number, label, copy_from: self.copy_from },
        }
    }
}
