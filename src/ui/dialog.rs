use super::action::Action;
use super::confirm_dialog::ConfirmDialog;
use super::entry_dialog::EntryDialog;
use super::semester_dialog::SemesterDialog;
use super::subject_dialog::SubjectDialog;

/// The dialog currently open on top of the page. At most one is open at a
/// time. Each variant carries the dialog's own editing state.
pub enum Dialog {
    Semester(SemesterDialog),
    Subject(SubjectDialog),
    Entry(EntryDialog),
    Confirm(ConfirmDialog),
}

impl Dialog {
    /// Draws the open dialog and collects what the user did in it.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        match self {
            Dialog::Semester(dialog) => dialog.show(ctx, actions),
            Dialog::Subject(dialog) => dialog.show(ctx, actions),
            Dialog::Entry(dialog) => dialog.show(ctx, actions),
            Dialog::Confirm(dialog) => dialog.show(ctx, actions),
        }
    }
}
