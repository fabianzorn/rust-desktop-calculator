//! Desktop window configuration and UI module composition.

mod formatting;
mod input;
mod programmer;
mod state;
mod view;

use eframe::egui;

use self::view::{CalculatorApp, STANDARD_MIN_WINDOW_SIZE, STANDARD_WINDOW_SIZE};

/// Creates and runs the native calculator window.
///
/// # Errors
///
/// Returns an [`eframe::Error`] if the native window cannot be created or the
/// application runtime fails.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(STANDARD_WINDOW_SIZE)
            .with_min_inner_size(STANDARD_MIN_WINDOW_SIZE)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Calculator",
        options,
        Box::new(|_creation_context| Ok(Box::new(CalculatorApp::default()))),
    )
}
