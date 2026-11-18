use super::action::Action;
use super::command::Command;
use super::store::Store;
use super::widgets::error_label;
use crate::db::{CategoryDraft, SubjectDraft};
use crate::domain::{
    CategoryData, CategoryId, CategoryKind, GradingKey, SemesterId, SubjectData, SubjectId,
    format_hundredths, parse_hundredths,
};

/// One editable line in the category table.
#[derive(Clone)]
struct CategoryRow {
    /// `None` for a category that is not stored yet.
    id: Option<CategoryId>,
    name: String,
    weight_percent: u32,
    kind: CategoryKind,
    /// Checkbox "only the best N entries count".
    limit_best: bool,
    best_n: u32,
    /// Stored entries; the kind of a category with entries is locked and
    /// removing it deletes them.
    entry_count: usize,
}

impl CategoryRow {
    fn new(name: &str, weight_percent: u32, kind: CategoryKind) -> CategoryRow {
        CategoryRow {
            id: None,
            name: name.to_string(),
            weight_percent,
            kind,
            limit_best: false,
            best_n: 1,
            entry_count: 0,
        }
    }

    fn from_stored(data: &CategoryData) -> CategoryRow {
        CategoryRow {
            id: Some(data.category.id),
            name: data.category.name.clone(),
            weight_percent: data.category.weight_percent,
            kind: data.category.kind,
            limit_best: data.category.best_n.is_some(),
            best_n: data.category.best_n.unwrap_or(1),
            entry_count: data.entries.len(),
        }
    }

    /// A copy for use in another subject: same settings, nothing stored.
    fn as_template(&self) -> CategoryRow {
        CategoryRow { id: None, entry_count: 0, ..self.clone() }
    }
}

/// Dialog for creating or editing a subject: name, categories with
/// weights, and the grading key.
pub struct SubjectDialog {
    /// `None` while creating a new subject.
    id: Option<SubjectId>,
    semester_id: SemesterId,
    name: String,
    rows: Vec<CategoryRow>,
    /// Entries that will be deleted because their category was removed.
    removed_entries: usize,
    key_known: bool,
    /// Thresholds for the grades 1–4 in percent, as typed by the user.
    thresholds: [String; 4],
    /// Other subjects whose categories can be taken over (name, rows).
    templates: Vec<(String, Vec<CategoryRow>)>,
}

impl SubjectDialog {
    /// Dialog for a new subject with a typical starting set of categories.
    pub fn create(store: &Store, semester_id: SemesterId) -> SubjectDialog {
        let templates = store
            .semesters()
            .iter()
            .flat_map(|semester| {
                semester.subjects.iter().filter(|s| !s.categories.is_empty()).map(|subject| {
                    let name = format!("{} – {}", subject.subject.name, semester.semester.display_name());
                    let rows = subject.categories.iter().map(CategoryRow::from_stored);
                    (name, rows.map(|row| row.as_template()).collect())
                })
            })
            .collect();
        SubjectDialog {
            id: None,
            semester_id,
            name: String::new(),
            rows: vec![
                CategoryRow::new("Schularbeit", 50, CategoryKind::Points),
                CategoryRow::new("Mitarbeit", 50, CategoryKind::Grade),
            ],
            removed_entries: 0,
            key_known: false,
            thresholds: threshold_texts(&GradingKey::default_unknown()),
            templates,
        }
    }

    /// Dialog for changing an existing subject.
    pub fn edit(data: &SubjectData) -> SubjectDialog {
        SubjectDialog {
            id: Some(data.subject.id),
            semester_id: data.subject.semester_id,
            name: data.subject.name.clone(),
            rows: data.categories.iter().map(CategoryRow::from_stored).collect(),
            removed_entries: 0,
            key_known: data.subject.key.known,
            thresholds: threshold_texts(&data.subject.key),
            templates: Vec::new(),
        }
    }

    /// Draws the dialog.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        egui::Modal::new(egui::Id::unique("subject_dialog")).show(ctx, |ui| {
            ui.set_width(620.0);
            ui.heading(if self.id.is_some() { "Fach bearbeiten" } else { "Neues Fach" });
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.add(egui::TextEdit::singleline(&mut self.name).desired_width(260.0));
            });

            ui.add_space(10.0);
            ui.strong("Kategorien");
            self.show_template_choice(ui);
            self.show_category_table(ui);

            ui.add_space(10.0);
            ui.strong("Notenschlüssel");
            self.show_key(ui);

            ui.add_space(12.0);
            let draft = self.draft();
            if let Err(message) = &draft {
                error_label(ui, message);
            }
            ui.horizontal(|ui| {
                let save = ui.add_enabled(draft.is_ok(), egui::Button::new("Speichern"));
                if let (true, Ok(draft)) = (save.clicked(), draft) {
                    let semester_id = self.semester_id;
                    actions.push(Action::Run(Command::SaveSubject { semester_id, draft }));
                }
                if ui.button("Abbrechen").clicked() {
                    actions.push(Action::CloseDialog);
                }
            });
        });
    }

    /// Drop-down "take categories from another subject" (new subjects only).
    fn show_template_choice(&mut self, ui: &mut egui::Ui) {
        if self.templates.is_empty() {
            return;
        }
        let mut chosen = None;
        ui.horizontal(|ui| {
            ui.label("Vorlage:");
            egui::ComboBox::from_id_salt("category_template")
                .selected_text("Kategorien übernehmen von …")
                .show_ui(ui, |ui| {
                    for (index, (name, _)) in self.templates.iter().enumerate() {
                        if ui.selectable_label(false, name).clicked() {
                            chosen = Some(index);
                        }
                    }
                });
        });
        if let Some(index) = chosen {
            self.rows = self.templates[index].1.clone();
        }
    }

    fn show_category_table(&mut self, ui: &mut egui::Ui) {
        let mut remove = None;
        egui::Grid::new("category_table").num_columns(5).spacing([10.0, 6.0]).show(ui, |ui| {
            for heading in ["Name", "Gewicht", "Typ", "Nur die besten …", ""] {
                ui.label(egui::RichText::new(heading).weak());
            }
            ui.end_row();

            for (index, row) in self.rows.iter_mut().enumerate() {
                ui.add_sized([160.0, 20.0], egui::TextEdit::singleline(&mut row.name));
                ui.add(egui::DragValue::new(&mut row.weight_percent).range(0..=100).suffix(" %"));

                // Changing the kind would make the stored entries invalid.
                ui.add_enabled_ui(row.entry_count == 0, |ui| {
                    egui::ComboBox::from_id_salt(("category_kind", index))
                        .selected_text(row.kind.label())
                        .show_ui(ui, |ui| {
                            for kind in [CategoryKind::Points, CategoryKind::Grade] {
                                ui.selectable_value(&mut row.kind, kind, kind.label());
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut row.limit_best, "");
                    if row.limit_best {
                        ui.add(egui::DragValue::new(&mut row.best_n).range(1..=99));
                        ui.label("zählen");
                    }
                });

                let hint = match row.entry_count {
                    0 => "Kategorie entfernen".to_string(),
                    count => format!("Kategorie entfernen – löscht beim Speichern auch {count} Einträge"),
                };
                if ui.button("Entfernen").on_hover_text(hint).clicked() {
                    remove = Some(index);
                }
                ui.end_row();
            }
        });

        if let Some(index) = remove {
            self.removed_entries += self.rows.remove(index).entry_count;
        }

        ui.horizontal(|ui| {
            if ui.button("+ Kategorie").clicked() {
                self.rows.push(CategoryRow::new("", 0, CategoryKind::Points));
            }
            ui.label(format!("Summe der Gewichte: {} %", self.weight_sum()));
        });
        if self.removed_entries > 0 {
            error_label(
                ui,
                &format!(
                    "Achtung: Beim Speichern werden {} Einträge der entfernten Kategorien gelöscht.",
                    self.removed_entries
                ),
            );
        }
    }

    fn show_key(&mut self, ui: &mut egui::Ui) {
        let threshold_field = |ui: &mut egui::Ui, label: &str, text: &mut String| {
            ui.label(label);
            ui.horizontal(|ui| {
                ui.label("über");
                ui.add(egui::TextEdit::singleline(text).desired_width(50.0));
                ui.label("%");
            });
            ui.end_row();
        };

        ui.checkbox(&mut self.key_known, "Ich kenne den echten Notenschlüssel für 1–3");
        egui::Grid::new("key_fields").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            if self.key_known {
                let [one, two, three, _] = &mut self.thresholds;
                threshold_field(ui, "Sehr gut (1):", one);
                threshold_field(ui, "Gut (2):", two);
                threshold_field(ui, "Befriedigend (3):", three);
            }
            threshold_field(ui, "Positiv / Genügend (4):", &mut self.thresholds[3]);
        });
        if !self.key_known {
            ui.label(
                egui::RichText::new(
                    "Ohne bekannten Schlüssel werden die Grenzen für 1–3 gleichmäßig über der \
                     Positiv-Grenze angenommen. Davon abhängige Noten sind mit ~ als geschätzt markiert.",
                )
                .weak(),
            );
        }
    }

    fn weight_sum(&self) -> u32 {
        self.rows.iter().map(|row| row.weight_percent).sum()
    }

    /// The grading key for the current input; `None` if a field is not a
    /// number.
    fn key(&self) -> Option<GradingKey> {
        let parse = |index: usize| parse_hundredths(&self.thresholds[index]);
        if self.key_known {
            Some(GradingKey {
                thresholds: [parse(0)?, parse(1)?, parse(2)?, parse(3)?],
                known: true,
            })
        } else {
            Some(GradingKey::assumed_for_pass_mark(parse(3)?))
        }
    }

    /// Converts the input into a saveable subject, or explains in German
    /// what still has to be fixed.
    fn draft(&self) -> Result<SubjectDraft, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("Bitte einen Namen für das Fach eingeben.".into());
        }
        if self.rows.is_empty() {
            return Err("Das Fach braucht mindestens eine Kategorie.".into());
        }
        if self.rows.iter().any(|row| row.name.trim().is_empty()) {
            return Err("Jede Kategorie braucht einen Namen.".into());
        }
        let sum = self.weight_sum();
        if sum != 100 {
            return Err(format!("Die Gewichte ergeben {sum} %, es müssen genau 100 % sein."));
        }
        let key = self
            .key()
            .filter(GradingKey::is_valid)
            .ok_or("Die Grenzen des Notenschlüssels müssen Zahlen unter 100 sein und von 1 bis 4 kleiner werden.")?;

        Ok(SubjectDraft {
            id: self.id,
            name: name.to_string(),
            key,
            categories: self
                .rows
                .iter()
                .map(|row| CategoryDraft {
                    id: row.id,
                    name: row.name.trim().to_string(),
                    weight_percent: row.weight_percent,
                    kind: row.kind,
                    best_n: row.limit_best.then_some(row.best_n),
                })
                .collect(),
        })
    }
}

/// The thresholds of a key as editable percent texts ("87,5").
fn threshold_texts(key: &GradingKey) -> [String; 4] {
    key.thresholds.map(format_hundredths)
}
