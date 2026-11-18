use jiff::Zoned;
use jiff::civil::Date;
use egui_extras::DatePickerButton;

use super::action::Action;
use super::command::Command;
use super::widgets::{error_label, grade_combo, optional_grade_combo};
use crate::domain::{Category, CategoryKind, Entry, EntryId, EntryValue, Grade, Points};

/// Dialog for adding or editing one entry of a category.
pub struct EntryDialog {
    /// `None` while adding a new entry.
    entry_id: Option<EntryId>,
    category: Category,
    date: Date,
    description: String,
    /// Input for grade categories.
    grade: Grade,
    /// Text inputs for points categories; parsed when saving.
    achieved: String,
    max: String,
    actual_grade: Option<Grade>,
}

impl EntryDialog {
    /// Dialog for a new entry dated today. `suggested_max` pre-fills the
    /// maximum points (usually the maximum of the previous entry).
    pub fn create(category: &Category, suggested_max: Option<Points>) -> EntryDialog {
        EntryDialog {
            entry_id: None,
            category: category.clone(),
            date: Zoned::now().date(),
            description: String::new(),
            grade: Grade::Three,
            achieved: String::new(),
            max: suggested_max.map_or(String::new(), |max| max.to_string()),
            actual_grade: None,
        }
    }

    /// Dialog for changing an existing entry.
    pub fn edit(category: &Category, entry: &Entry) -> EntryDialog {
        let mut dialog = EntryDialog::create(category, None);
        dialog.entry_id = Some(entry.id);
        dialog.date = entry.date;
        dialog.description = entry.description.clone();
        match entry.value {
            EntryValue::Grade(grade) => dialog.grade = grade,
            EntryValue::Points { achieved, max, actual_grade } => {
                dialog.achieved = achieved.to_string();
                dialog.max = max.to_string();
                dialog.actual_grade = actual_grade;
            }
        }
        dialog
    }

    /// Draws the dialog.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        egui::Modal::new(egui::Id::unique("entry_dialog")).show(ctx, |ui| {
            ui.set_width(400.0);
            let verb = if self.entry_id.is_some() { "bearbeiten" } else { "hinzufügen" };
            ui.heading(format!("{}: Eintrag {verb}", self.category.name));
            ui.add_space(8.0);

            egui::Grid::new("entry_fields").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                ui.label("Datum:");
                ui.add(DatePickerButton::new(&mut self.date).format("%d.%m.%Y"));
                ui.end_row();

                ui.label("Beschreibung:");
                ui.add(egui::TextEdit::singleline(&mut self.description).hint_text("optional"));
                ui.end_row();

                match self.category.kind {
                    CategoryKind::Grade => {
                        ui.label("Note:");
                        grade_combo(ui, "entry_grade", &mut self.grade);
                        ui.end_row();
                    }
                    CategoryKind::Points => self.show_points_fields(ui),
                }
            });

            ui.add_space(12.0);
            let value = self.value();
            if let Err(message) = &value {
                error_label(ui, message);
            }
            ui.horizontal(|ui| {
                let save = ui.add_enabled(value.is_ok(), egui::Button::new("Speichern"));
                if let (true, Ok(value)) = (save.clicked(), value) {
                    actions.push(Action::Run(self.command(value)));
                }
                if ui.button("Abbrechen").clicked() {
                    actions.push(Action::CloseDialog);
                }
            });
        });
    }

    /// The grid rows for a points entry.
    fn show_points_fields(&mut self, ui: &mut egui::Ui) {
        ui.label("Punkte:");
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.achieved).desired_width(60.0));
            ui.label("von");
            ui.add(egui::TextEdit::singleline(&mut self.max).desired_width(60.0));
        });
        ui.end_row();

        ui.label("Tatsächliche Note:");
        optional_grade_combo(ui, "entry_actual_grade", &mut self.actual_grade);
        ui.end_row();

        ui.label("");
        ui.label(
            egui::RichText::new("Die Note der Lehrkraft hat Vorrang vor dem Notenschlüssel.").weak(),
        );
        ui.end_row();
    }

    /// Converts the input into an entry value, or explains what is wrong.
    fn value(&self) -> Result<EntryValue, &'static str> {
        match self.category.kind {
            CategoryKind::Grade => Ok(EntryValue::Grade(self.grade)),
            CategoryKind::Points => {
                const HELP: &str = "Punkte bitte als Zahl mit höchstens zwei Nachkommastellen eingeben.";
                let achieved = Points::parse(&self.achieved).ok_or(HELP)?;
                let max = Points::parse(&self.max).ok_or(HELP)?;
                if max.hundredths() == 0 {
                    return Err("Die maximale Punktezahl muss größer als 0 sein.");
                }
                Ok(EntryValue::Points { achieved, max, actual_grade: self.actual_grade })
            }
        }
    }

    /// The database change for a valid input.
    fn command(&self, value: EntryValue) -> Command {
        let description = self.description.trim().to_string();
        match self.entry_id {
            Some(id) => Command::UpdateEntry(Entry {
                id,
                category_id: self.category.id,
                date: self.date,
                description,
                value,
            }),
            None => Command::AddEntry {
                category_id: self.category.id,
                date: self.date,
                description,
                value,
            },
        }
    }
}
