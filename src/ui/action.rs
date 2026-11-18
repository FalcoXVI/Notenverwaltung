use std::path::PathBuf;

use super::command::Command;
use super::dialog::Dialog;
use super::screen::Screen;
use crate::domain::SemesterId;

/// Something the user did in a view or dialog. Views push actions into a
/// list while drawing; the app handles them after the frame is drawn.
pub enum Action {
    /// Show another page.
    Navigate(Screen),
    /// Show another semester in the overview.
    SelectSemester(SemesterId),
    /// Open a dialog on top of the current page.
    OpenDialog(Dialog),
    /// Close the open dialog without saving.
    CloseDialog,
    /// Change the stored data.
    Run(Command),
    /// Ask "are you sure?" and run the command only after a yes.
    Confirm { message: String, command: Command },
    /// Write all grades into a CSV file.
    ExportCsv(PathBuf),
    /// Write a backup of the database to the chosen file.
    Backup(PathBuf),
}
