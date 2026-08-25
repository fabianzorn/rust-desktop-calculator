//! Desktop calculator application built with `eframe` and `egui`.

#![warn(missing_docs)]

mod calculator;
mod desktop_ui;

/// Starts the desktop application and reports startup failures to stderr.
fn main() {
    if let Err(error) = desktop_ui::run() {
        eprintln!("Error while starting the Desktop-UI: {error}");
    }
}
