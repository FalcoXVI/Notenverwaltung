use jiff::Zoned;

use super::action::Action;
use super::command::Command;
use super::store::Store;
use super::widgets::error_label;
use crate::calc::TrafficThresholds;
use crate::db::{backup_dir, database_file};
use crate::domain::{Fraction, format_hundredths, fraction, parse_hundredths};

/// Input state of the settings page.
pub struct SettingsState {
    /// Traffic light limits as typed by the user.
    green_max: String,
    yellow_max: String,
}

impl SettingsState {
    /// Fills the inputs with the stored limits.
    pub fn new(thresholds: &TrafficThresholds) -> SettingsState {
        let text = |limit: Fraction| format_hundredths((limit * 100).round().to_integer());
        SettingsState {
            green_max: text(thresholds.green_max),
            yellow_max: text(thresholds.yellow_max),
        }
    }

    /// The limits for the current input, or what is wrong with it.
    fn thresholds(&self) -> Result<TrafficThresholds, &'static str> {
        const HELP: &str = "Bitte zwei Zahlen zwischen 1 und 5 eingeben, Grün kleiner als Gelb.";
        let green = parse_hundredths(&self.green_max).ok_or(HELP)?;
        let yellow = parse_hundredths(&self.yellow_max).ok_or(HELP)?;
        if !(100 <= green && green < yellow && yellow <= 500) {
            return Err(HELP);
        }
        Ok(TrafficThresholds {
            green_max: fraction(green, 100),
            yellow_max: fraction(yellow, 100),
        })
    }
}

/// Draws the settings page: traffic light limits, export and backups.
pub fn show(ui: &mut egui::Ui, store: &Store, state: &mut SettingsState, actions: &mut Vec<Action>) {
    ui.heading("Einstellungen & Daten");
    ui.add_space(8.0);

    ui.strong("Ampel");
    show_traffic_settings(ui, store, state, actions);

    ui.add_space(16.0);
    ui.strong("Export");
    ui.label("Alle Einträge aller Semester als CSV-Datei (für Excel).");
    if ui.button("Als CSV exportieren …").clicked() {
        let chosen = rfd::FileDialog::new()
            .set_title("CSV-Export speichern")
            .set_file_name("noten.csv")
            .add_filter("CSV-Datei", &["csv"])
            .save_file();
        if let Some(path) = chosen {
            actions.push(Action::ExportCsv(path));
        }
    }

    ui.add_space(16.0);
    ui.strong("Sicherung");
    show_backup_section(ui, actions);
}

fn show_traffic_settings(ui: &mut egui::Ui, store: &Store, state: &mut SettingsState, actions: &mut Vec<Action>) {
    egui::Grid::new("traffic_settings").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
        ui.label("Grün bis einschließlich Schnitt:");
        ui.add(egui::TextEdit::singleline(&mut state.green_max).desired_width(50.0));
        ui.end_row();
        ui.label("Gelb bis einschließlich Schnitt:");
        ui.add(egui::TextEdit::singleline(&mut state.yellow_max).desired_width(50.0));
        ui.end_row();
    });
    ui.label(egui::RichText::new("Alles darüber ist rot.").weak());

    let thresholds = state.thresholds();
    if let Err(message) = thresholds {
        error_label(ui, message);
    }
    let changed = thresholds.is_ok_and(|t| t != *store.thresholds());
    if ui.add_enabled(changed, egui::Button::new("Ampel speichern")).clicked()
        && let Ok(thresholds) = thresholds
    {
        actions.push(Action::Run(Command::SaveTrafficThresholds(thresholds)));
    }
}

fn show_backup_section(ui: &mut egui::Ui, actions: &mut Vec<Action>) {
    ui.label(format!("Datenbank: {}", database_file().display()));
    ui.label(format!(
        "Bei jedem Start wird automatisch gesichert (die letzten 10 bleiben): {}",
        backup_dir().display()
    ));
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("Sicherung erstellen …").clicked() {
            let name = format!("noten-sicherung-{}.db", Zoned::now().strftime("%Y-%m-%d"));
            let chosen = rfd::FileDialog::new()
                .set_title("Sicherung speichern")
                .set_file_name(name)
                .add_filter("Datenbank-Sicherung", &["db"])
                .save_file();
            if let Some(path) = chosen {
                actions.push(Action::Backup(path));
            }
        }
        if ui.button("Sicherung wiederherstellen …").clicked() {
            let chosen = rfd::FileDialog::new()
                .set_title("Sicherung auswählen")
                .add_filter("Datenbank-Sicherung", &["db"])
                .pick_file();
            if let Some(path) = chosen {
                actions.push(Action::Confirm {
                    message: format!(
                        "Alle aktuellen Daten werden durch die Sicherung „{}“ ersetzt. \
                         Der jetzige Stand wird vorher automatisch gesichert.",
                        path.display()
                    ),
                    command: Command::Restore(path),
                });
            }
        }
        if ui.button("Ordner der automatischen Sicherungen öffnen").clicked() {
            // Best effort: if Explorer cannot be started nothing is lost.
            let _ = std::fs::create_dir_all(backup_dir());
            let _ = std::process::Command::new("explorer").arg(backup_dir()).spawn();
        }
    });
}
