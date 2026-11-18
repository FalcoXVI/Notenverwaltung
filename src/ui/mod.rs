//! Graphical user interface built with egui/eframe.
//!
//! egui is an "immediate mode" GUI: there are no widget objects that live
//! on. Instead the whole window is described again for every frame from
//! the current data. The flow per frame is:
//!
//! 1. views and dialogs read the [`store::Store`] and draw themselves,
//! 2. what the user did is collected as a list of [`action::Action`]s,
//! 3. [`app::App`] applies the actions (navigate, save to database, …).

mod action;
mod app;
mod command;
mod confirm_dialog;
mod dialog;
mod entry_dialog;
mod open_negative_view;
mod overview;
mod screen;
mod semester_dialog;
mod settings_view;
mod store;
mod subject_dialog;
mod subject_view;
mod what_if_panel;
mod widgets;

use app::App;
use store::Store;

/// Opens the database and runs the window until the user closes it.
///
/// Returns a German error message if the program cannot start.
pub fn run() -> Result<(), String> {
    let (store, backup_warning) = Store::open().map_err(|error| error.to_string())?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1150.0, 760.0])
            .with_min_inner_size([860.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Notenverwaltung",
        options,
        Box::new(move |context| {
            // Slightly larger than egui's default for comfortable reading.
            context.egui_ctx.set_zoom_factor(1.15);
            Ok(Box::new(App::new(store, backup_warning)))
        }),
    )
    .map_err(|error| format!("Das Fenster konnte nicht geöffnet werden: {error}"))
}
