use super::action::Action;
use super::command::Command;
use super::confirm_dialog::ConfirmDialog;
use super::dialog::Dialog;
use super::open_negative_view::{self, open_negative_subjects};
use super::overview;
use super::screen::Screen;
use super::settings_view::{self, SettingsState};
use super::store::Store;
use super::subject_view;
use super::what_if_panel::WhatIfState;
use super::widgets::RED;
use crate::db::DbError;
use crate::domain::SemesterId;

/// Message in the status bar at the bottom of the window.
enum Status {
    None,
    Info(String),
    Error(String),
}

/// The application window: owns the data and all GUI state.
pub struct App {
    store: Store,
    screen: Screen,
    /// Semester shown in the overview.
    semester: Option<SemesterId>,
    dialog: Option<Dialog>,
    what_if: WhatIfState,
    settings: SettingsState,
    status: Status,
}

impl App {
    /// Creates the window state, starting on the newest semester.
    pub fn new(store: Store, backup_warning: Option<DbError>) -> App {
        let status = match backup_warning {
            Some(error) => Status::Error(format!("Automatische Sicherung fehlgeschlagen. {error}")),
            None => Status::None,
        };
        App {
            semester: store.semesters().last().map(|s| s.semester.id),
            settings: SettingsState::new(store.thresholds()),
            store,
            screen: Screen::Overview,
            dialog: None,
            what_if: WhatIfState::default(),
            status,
        }
    }

    /// Makes sure the selected semester and the shown subject still exist
    /// (they may have been deleted or replaced by a restore).
    fn repair_selection(&mut self) {
        let semester_exists = self.semester.is_some_and(|id| self.store.semester(id).is_some());
        if !semester_exists {
            self.semester = self.store.semesters().last().map(|s| s.semester.id);
        }
        if let Screen::Subject(id) = self.screen
            && self.store.subject(id).is_none()
        {
            self.screen = Screen::Overview;
        }
    }

    /// The row of page links at the top.
    fn show_navigation(&self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let open_count = open_negative_subjects(&self.store).len();
            let on_overview = matches!(self.screen, Screen::Overview | Screen::Subject(_));
            let pages = [
                ("Übersicht".to_string(), Screen::Overview, on_overview),
                (
                    format!("Offene negative Fächer ({open_count})"),
                    Screen::OpenNegative,
                    self.screen == Screen::OpenNegative,
                ),
                ("Einstellungen & Daten".to_string(), Screen::Settings, self.screen == Screen::Settings),
            ];
            for (title, screen, active) in pages {
                if ui.selectable_label(active, title).clicked() {
                    actions.push(Action::Navigate(screen));
                }
            }
        });
        ui.add_space(4.0);
    }

    fn show_status(&self, ui: &mut egui::Ui) {
        match &self.status {
            Status::None => ui.label(""),
            Status::Info(message) => ui.label(egui::RichText::new(message).weak()),
            Status::Error(message) => ui.colored_label(RED, message),
        };
    }

    /// Applies one user action after the frame has been drawn.
    fn handle(&mut self, action: Action) {
        match action {
            Action::Navigate(screen) => self.navigate(screen),
            Action::SelectSemester(id) => self.semester = Some(id),
            Action::OpenDialog(dialog) => self.dialog = Some(dialog),
            Action::CloseDialog => self.dialog = None,
            Action::Confirm { message, command } => {
                self.dialog = Some(Dialog::Confirm(ConfirmDialog::new(message, command)));
            }
            Action::Run(command) => self.run(command),
            Action::ExportCsv(path) => {
                self.status = match self.store.export_csv(&path) {
                    Ok(()) => Status::Info(format!("Exportiert nach {}", path.display())),
                    Err(error) => Status::Error(format!("Export fehlgeschlagen: {error}")),
                };
            }
            Action::Backup(path) => {
                self.status = match self.store.backup_to(&path) {
                    Ok(()) => Status::Info(format!("Sicherung gespeichert: {}", path.display())),
                    Err(error) => Status::Error(format!("Sicherung fehlgeschlagen. {error}")),
                };
            }
        }
    }

    fn navigate(&mut self, screen: Screen) {
        match screen {
            Screen::Subject(id) => {
                // Start with fresh what-if inputs and remember the semester
                // so that "back" leads to the right overview.
                self.what_if = WhatIfState::default();
                if let Some((semester, _)) = self.store.subject(id) {
                    self.semester = Some(semester.semester.id);
                }
            }
            Screen::Settings => self.settings = SettingsState::new(self.store.thresholds()),
            Screen::Overview | Screen::OpenNegative => {}
        }
        self.screen = screen;
    }

    /// Writes a change to the database and reports the outcome.
    fn run(&mut self, command: Command) {
        let message = command.success_message();
        let creates_semester = matches!(command, Command::CreateSemester { .. });
        let replaces_everything = matches!(command, Command::Restore(_));

        match self.store.execute(command) {
            Ok(()) => {
                self.dialog = None;
                self.status = Status::Info(message.to_string());
                if creates_semester {
                    // Jump to the new semester: it has the highest id.
                    let newest = self.store.semesters().iter().map(|s| s.semester.id).max_by_key(|id| id.0);
                    self.semester = newest;
                }
                if replaces_everything {
                    self.settings = SettingsState::new(self.store.thresholds());
                }
            }
            // The dialog stays open so that no input is lost.
            Err(error) => self.status = Status::Error(error.to_string()),
        }
    }
}

impl eframe::App for App {
    /// Called by eframe for every frame: draws the whole window.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.repair_selection();
        let mut actions = Vec::new();

        egui::Panel::top("navigation").show(ui, |ui| self.show_navigation(ui, &mut actions));
        egui::Panel::bottom("status").show(ui, |ui| self.show_status(ui));
        egui::CentralPanel::default().show(ui, |ui| match self.screen {
            Screen::Overview => overview::show(ui, &self.store, self.semester, &mut actions),
            Screen::Subject(id) => {
                subject_view::show(ui, &self.store, id, &mut self.what_if, &mut actions);
            }
            Screen::OpenNegative => open_negative_view::show(ui, &self.store, &mut actions),
            Screen::Settings => {
                settings_view::show(ui, &self.store, &mut self.settings, &mut actions);
            }
        });

        if let Some(dialog) = &mut self.dialog {
            dialog.show(ui.ctx(), &mut actions);
        }
        for action in actions {
            self.handle(action);
        }
    }
}
