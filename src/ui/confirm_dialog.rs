use super::action::Action;
use super::command::Command;

/// "Are you sure?" dialog shown before anything is deleted or overwritten.
pub struct ConfirmDialog {
    message: String,
    command: Command,
}

impl ConfirmDialog {
    /// Creates a dialog that runs `command` if the user confirms.
    pub fn new(message: String, command: Command) -> ConfirmDialog {
        ConfirmDialog { message, command }
    }

    /// Draws the dialog.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        egui::Modal::new(egui::Id::unique("confirm_dialog")).show(ctx, |ui| {
            ui.set_max_width(420.0);
            ui.heading("Bist du sicher?");
            ui.add_space(6.0);
            ui.label(&self.message);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Ja, fortfahren").clicked() {
                    actions.push(Action::Run(self.command.clone()));
                }
                if ui.button("Abbrechen").clicked() {
                    actions.push(Action::CloseDialog);
                }
            });
        });
    }
}
