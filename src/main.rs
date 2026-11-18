// In release builds, do not open a console window next to the GUI.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Program entry point. All logic lives in the library crate (`lib.rs`);
/// this file only starts the GUI and reports a failed start.
fn main() {
    if let Err(message) = notenverwaltung::ui::run() {
        // Without a console the user would otherwise see nothing at all.
        rfd::MessageDialog::new()
            .set_title("Notenverwaltung")
            .set_description(message)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}
