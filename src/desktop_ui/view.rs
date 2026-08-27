//! egui widgets, layout, and visual styling for the calculator.

use eframe::egui::{self, Align, Button, Color32, Layout, RichText, Vec2};

use crate::calculator::{AngleMode, MathematicalConstant, Operator, UnaryOperator};

use super::formatting::{display_expression, display_size};
use super::input::{Key, copy_result_requested, keyboard_keys};
use super::state::CalculatorState;

const MIN_CALCULATOR_WIDTH: f32 = 300.0;
const MAX_CALCULATOR_WIDTH: f32 = 420.0;
const BUTTON_GAP: f32 = 9.0;
const BUTTON_HEIGHT: f32 = 52.0;
const BUTTON_ROW_GAP: f32 = 6.0;

/// Connects the calculator state to the native egui application lifecycle.
#[derive(Default)]
pub(super) struct CalculatorApp {
    state: CalculatorState,
}

impl eframe::App for CalculatorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().set_visuals(egui::Visuals::dark());

        if copy_result_requested(ui.ctx()) {
            self.copy_result(ui.ctx());
        }

        for key in keyboard_keys(ui.ctx()) {
            self.state.handle_key(key);
        }

        egui::Frame::default()
            .fill(Color32::from_rgb(17, 22, 29))
            .show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space(20.0);
                    self.show_calculator(ui);
                });
            });
    }
}

impl CalculatorApp {
    /// Copies the displayed result unless the calculator currently shows an error.
    fn copy_result(&self, context: &egui::Context) {
        if !self.state.has_error() {
            context.copy_text(self.state.display().to_owned());
        }
    }

    /// Renders the title, angle-mode status, and copy action.
    fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Calculator")
                    .color(Color32::from_rgb(244, 248, 251))
                    .size(20.0)
                    .strong(),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let copy = ui
                    .add_enabled(
                        !self.state.has_error(),
                        Button::new(
                            RichText::new("COPY")
                                .color(Color32::from_rgb(220, 232, 242))
                                .size(12.0),
                        )
                        .fill(Color32::from_rgb(49, 61, 74))
                        .corner_radius(6.0),
                    )
                    .on_hover_text("Copy result (Ctrl+C / Cmd+C)");
                if copy.clicked() {
                    self.copy_result(ui.ctx());
                }

                ui.label(
                    RichText::new(format!("ANGLE: {}", self.state.angle_mode().label()))
                        .color(Color32::from_rgb(115, 183, 235))
                        .size(12.0)
                        .strong(),
                )
                .on_hover_text("Active angle mode (M to switch)");

                if self.state.has_memory() {
                    ui.label(
                        RichText::new("M")
                            .color(Color32::from_rgb(126, 214, 168))
                            .size(12.0)
                            .strong(),
                    )
                    .on_hover_text("Calculator memory contains a value");
                }
            });
        });
    }

    /// Renders the calculator container and all of its sections.
    fn show_calculator(&mut self, ui: &mut egui::Ui) {
        let calculator_width =
            (ui.available_width() - 40.0).clamp(MIN_CALCULATOR_WIDTH, MAX_CALCULATOR_WIDTH);

        egui::Frame::default()
            .fill(Color32::from_rgb(35, 43, 53))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(58, 70, 84)))
            .corner_radius(14.0)
            .inner_margin(18.0)
            .show(ui, |ui| {
                ui.set_width(calculator_width);
                self.show_header(ui);
                ui.add_space(14.0);
                self.show_display(ui);
                ui.add_space(14.0);
                self.show_buttons(ui);
            });
    }

    /// Renders the current expression and result display.
    fn show_display(&self, ui: &mut egui::Ui) {
        egui::Frame::default()
            .fill(Color32::from_rgb(11, 17, 24))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(61, 76, 90)))
            .corner_radius(10.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(90.0);
                ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                    ui.label(
                        RichText::new(display_expression(self.state.expression()))
                            .color(Color32::from_rgb(143, 164, 183))
                            .size(14.0),
                    );

                    let display_color = if self.state.has_error() {
                        Color32::from_rgb(255, 189, 189)
                    } else {
                        Color32::from_rgb(247, 251, 255)
                    };

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(self.state.display())
                            .color(display_color)
                            .size(display_size(self.state.display()))
                            .strong(),
                    );
                });
            });
    }

    /// Renders all calculator button rows.
    fn show_buttons(&mut self, ui: &mut egui::Ui) {
        self.show_button_row(
            ui,
            &[
                Key::MemoryClear,
                Key::MemoryRecall,
                Key::MemoryAdd,
                Key::MemorySubtract,
                Key::OpenParenthesis,
                Key::CloseParenthesis,
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::UnaryOperator(UnaryOperator::Sine),
                Key::UnaryOperator(UnaryOperator::Cosine),
                Key::UnaryOperator(UnaryOperator::Tangent),
                Key::ToggleAngleMode,
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::UnaryOperator(UnaryOperator::LogarithmBase10),
                Key::UnaryOperator(UnaryOperator::NaturalLogarithm),
                Key::UnaryOperator(UnaryOperator::Exponential),
                Key::Operator(Operator::Power),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::UnaryOperator(UnaryOperator::Reciprocal),
                Key::UnaryOperator(UnaryOperator::Factorial),
                Key::UnaryOperator(UnaryOperator::SquareRoot),
                Key::UnaryOperator(UnaryOperator::Square),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Constant(MathematicalConstant::Pi),
                Key::Constant(MathematicalConstant::Euler),
                Key::UnaryOperator(UnaryOperator::ToggleSign),
                Key::UnaryOperator(UnaryOperator::Percent),
                Key::Clear,
                Key::Backspace,
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('7'),
                Key::Number('8'),
                Key::Number('9'),
                Key::Operator(Operator::Divide),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('4'),
                Key::Number('5'),
                Key::Number('6'),
                Key::Operator(Operator::Multiply),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::Number('3'),
                Key::Operator(Operator::Subtract),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('0'),
                Key::Decimal,
                Key::Equals,
                Key::Operator(Operator::Add),
            ],
        );
    }

    /// Renders one row of buttons and forwards clicks to the state.
    fn show_button_row(&mut self, ui: &mut egui::Ui, keys: &[Key]) {
        let gaps_width = BUTTON_GAP * keys.len().saturating_sub(1) as f32;
        let button_width = (ui.available_width() - gaps_width) / keys.len() as f32;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(BUTTON_GAP);

            for key in keys {
                let response = ui
                    .add_sized(
                        [button_width, BUTTON_HEIGHT],
                        button_for(
                            *key,
                            self.state.is_active_operator(*key),
                            self.state.angle_mode(),
                        ),
                    )
                    .on_hover_text(tooltip_for(*key));
                if response.clicked() {
                    self.state.handle_key(*key);
                }
            }
        });
        ui.add_space(BUTTON_ROW_GAP);
    }
}

/// Creates a styled button for a calculator action.
fn button_for(key: Key, active: bool, angle_mode: AngleMode) -> Button<'static> {
    let (label, color, text_color) = match key {
        Key::Number(number) => (
            number.to_string(),
            key_color(),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Decimal => (
            ".".to_owned(),
            key_color(),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Operator(operator) => (
            if operator == Operator::Power {
                "x^y".to_owned()
            } else {
                operator.symbol().to_string()
            },
            if active {
                Color32::from_rgb(244, 248, 251)
            } else {
                Color32::from_rgb(238, 135, 65)
            },
            if active {
                Color32::from_rgb(28, 36, 45)
            } else {
                Color32::WHITE
            },
        ),
        Key::UnaryOperator(operator) => (
            match operator {
                UnaryOperator::ToggleSign => "±",
                UnaryOperator::Percent => "%",
                UnaryOperator::SquareRoot => "√",
                UnaryOperator::Square => "x²",
                UnaryOperator::Sine => "sin",
                UnaryOperator::Cosine => "cos",
                UnaryOperator::Tangent => "tan",
                UnaryOperator::LogarithmBase10 => "log₁₀",
                UnaryOperator::NaturalLogarithm => "ln",
                UnaryOperator::Exponential => "e^x",
                UnaryOperator::Reciprocal => "1/x",
                UnaryOperator::Factorial => "x!",
            }
            .to_owned(),
            Color32::from_rgb(67, 80, 95),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Constant(constant) => (
            constant.symbol().to_owned(),
            Color32::from_rgb(48, 100, 145),
            Color32::WHITE,
        ),
        Key::MemoryClear | Key::MemoryRecall | Key::MemoryAdd | Key::MemorySubtract => (
            match key {
                Key::MemoryClear => "MC",
                Key::MemoryRecall => "MR",
                Key::MemoryAdd => "M+",
                Key::MemorySubtract => "M−",
                _ => unreachable!(),
            }
            .to_owned(),
            Color32::from_rgb(55, 91, 112),
            Color32::from_rgb(226, 241, 249),
        ),
        Key::OpenParenthesis | Key::CloseParenthesis => (
            if key == Key::OpenParenthesis {
                "(".to_owned()
            } else {
                ")".to_owned()
            },
            Color32::from_rgb(67, 80, 95),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Equals => (
            "=".to_owned(),
            Color32::from_rgb(42, 146, 103),
            Color32::WHITE,
        ),
        Key::Backspace => (
            "DEL".to_owned(),
            Color32::from_rgb(67, 80, 95),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Clear => (
            "AC".to_owned(),
            Color32::from_rgb(150, 70, 78),
            Color32::from_rgb(255, 245, 245),
        ),
        Key::ToggleAngleMode => (
            angle_mode.label().to_owned(),
            Color32::from_rgb(48, 100, 145),
            Color32::WHITE,
        ),
    };

    Button::new(RichText::new(label).color(text_color).size(20.0).strong())
        .fill(color)
        .stroke(egui::Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 24),
        ))
        .corner_radius(10.0)
}

/// Returns a short description and keyboard shortcut for a calculator key.
fn tooltip_for(key: Key) -> String {
    match key {
        Key::Number(number) => format!("Enter {number} ({number})"),
        Key::Decimal => "Decimal point (. or ,)".to_owned(),
        Key::Operator(operator) => match operator {
            Operator::Add => "Addition (+)",
            Operator::Subtract => "Subtraction (-)",
            Operator::Multiply => "Multiplication (*)",
            Operator::Divide => "Division (/)",
            Operator::Power => "Power (^)",
        }
        .to_owned(),
        Key::UnaryOperator(operator) => match operator {
            UnaryOperator::ToggleSign => "Toggle sign (N)",
            UnaryOperator::Percent => "Percentage (%)",
            UnaryOperator::SquareRoot => "Square root (R)",
            UnaryOperator::Square => "Square (S)",
            UnaryOperator::Sine => "Sine (I)",
            UnaryOperator::Cosine => "Cosine (C)",
            UnaryOperator::Tangent => "Tangent (T)",
            UnaryOperator::LogarithmBase10 => "Base-10 logarithm (O)",
            UnaryOperator::NaturalLogarithm => "Natural logarithm (L)",
            UnaryOperator::Exponential => "Exponential e^x (E)",
            UnaryOperator::Reciprocal => "Reciprocal (V)",
            UnaryOperator::Factorial => "Factorial (F)",
        }
        .to_owned(),
        Key::Constant(constant) => match constant {
            MathematicalConstant::Pi => "Enter pi (P)",
            MathematicalConstant::Euler => "Enter Euler's number (K)",
        }
        .to_owned(),
        Key::MemoryClear => "Clear memory (Ctrl+L)".to_owned(),
        Key::MemoryRecall => "Recall memory (Ctrl+R)".to_owned(),
        Key::MemoryAdd => "Add display to memory (Ctrl+P)".to_owned(),
        Key::MemorySubtract => "Subtract display from memory (Ctrl+Q)".to_owned(),
        Key::OpenParenthesis => "Open parenthesis — key: (".to_owned(),
        Key::CloseParenthesis => "Close parenthesis — key: )".to_owned(),
        Key::Equals => "Calculate result (Enter or =)".to_owned(),
        Key::Backspace => "Delete last digit (Backspace)".to_owned(),
        Key::Clear => "Clear calculator (Escape or Delete)".to_owned(),
        Key::ToggleAngleMode => "Switch angle mode (M)".to_owned(),
    }
}

/// Returns the background color used by number and decimal buttons.
fn key_color() -> Color32 {
    Color32::from_rgb(49, 61, 74)
}
