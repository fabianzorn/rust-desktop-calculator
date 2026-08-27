//! Desktop window configuration and UI module composition.

mod formatting;
mod input;
mod state;
mod view;

use eframe::egui;

use self::view::CalculatorApp;

const MIN_WINDOW_SIZE: [f32; 2] = [340.0, 790.0];
const START_WINDOW_SIZE: [f32; 2] = [380.0, 850.0];

/// Creates and runs the native calculator window.
///
/// # Errors
///
/// Returns an [`eframe::Error`] if the native window cannot be created or the
/// application runtime fails.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(START_WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Calculator",
        options,
        Box::new(|_creation_context| Ok(Box::new(CalculatorApp::default()))),
    )
}
