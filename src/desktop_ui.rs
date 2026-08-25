use eframe::egui::{self, Align, Button, Color32, Layout, RichText, Vec2};

use crate::calculator::{CalculationError, Operator, calculate};

const MIN_WINDOW_SIZE: [f32; 2] = [340.0, 560.0];
const START_WINDOW_SIZE: [f32; 2] = [380.0, 620.0];
const MIN_CALCULATOR_WIDTH: f32 = 300.0;
const MAX_CALCULATOR_WIDTH: f32 = 420.0;
const BUTTON_GAP: f32 = 9.0;
const BUTTON_HEIGHT: f32 = 58.0;

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

struct CalculatorApp {
    first_value: Option<f64>,
    operator: Option<Operator>,
    display: String,
    expression: String,
    waiting_for_second_value: bool,
    has_error: bool,
}

impl Default for CalculatorApp {
    fn default() -> Self {
        Self {
            first_value: None,
            operator: None,
            display: "0".to_owned(),
            expression: String::new(),
            waiting_for_second_value: false,
            has_error: false,
        }
    }
}

impl eframe::App for CalculatorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().set_visuals(egui::Visuals::dark());

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

    fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Calculator")
                    .color(Color32::from_rgb(244, 248, 251))
                    .size(20.0)
                    .strong(),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new("egui")
                        .color(Color32::from_rgb(133, 151, 169))
                        .size(12.0),
                );
            });
        });
    }

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
                        RichText::new(display_expression(&self.expression))
                            .color(Color32::from_rgb(143, 164, 183))
                            .size(14.0),
                    );

                    let display_color = if self.has_error {
                        Color32::from_rgb(255, 189, 189)
                    } else {
                        Color32::from_rgb(247, 251, 255)
                    };

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(&self.display)
                            .color(display_color)
                            .size(display_size(&self.display))
                            .strong(),
                    );
                });
            });
    }

    fn show_buttons(&mut self, ui: &mut egui::Ui) {
        self.show_button_row(
            ui,
            &[
                Key::Clear,
                Key::Backspace,
                Key::Decimal,
                Key::Operator(Operator::Divide),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('7'),
                Key::Number('8'),
                Key::Number('9'),
                Key::Operator(Operator::Multiply),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('4'),
                Key::Number('5'),
                Key::Number('6'),
                Key::Operator(Operator::Subtract),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::Number('3'),
                Key::Operator(Operator::Add),
            ],
        );
        self.show_button_row(ui, &[Key::Number('0'), Key::Equals]);
    }

    fn show_button_row(&mut self, ui: &mut egui::Ui, keys: &[Key]) {
        let single_width = (ui.available_width() - (BUTTON_GAP * 3.0)) / 4.0;
        let double_width = (single_width * 2.0) + BUTTON_GAP;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(BUTTON_GAP);

            for key in keys {
                let width = if matches!(key, Key::Number('0') | Key::Equals) {
                    double_width
                } else {
                    single_width
                };

                if ui
                    .add_sized(
                        [width, BUTTON_HEIGHT],
                        button_for(*key, self.is_active_operator(*key)),
                    )
                    .clicked()
                {
                    self.handle_key(*key);
                }
            }
        });
        ui.add_space(8.0);
    }

    fn handle_key(&mut self, key: Key) {
        match key {
            Key::Number(number) => self.append_number(number),
            Key::Decimal => self.append_decimal(),
            Key::Operator(operator) => self.choose_operator(operator),
            Key::Equals => self.calculate_result(),
            Key::Backspace => self.backspace(),
            Key::Clear => self.clear(),
        }
    }

    fn is_active_operator(&self, key: Key) -> bool {
        matches!(
            (key, self.operator, self.waiting_for_second_value),
            (Key::Operator(key_operator), Some(current_operator), true)
                if key_operator == current_operator
        )
    }

    fn append_number(&mut self, number: char) {
        self.clear_error();

        if self.waiting_for_second_value {
            self.display = number.to_string();
            self.waiting_for_second_value = false;
        } else if self.display == "0" {
            self.display = number.to_string();
        } else {
            self.display.push(number);
        }

        self.update_expression();
    }

    fn append_decimal(&mut self) {
        self.clear_error();

        if self.waiting_for_second_value {
            self.display = "0.".to_owned();
            self.waiting_for_second_value = false;
        } else if !self.display.contains('.') {
            self.display.push('.');
        }

        self.update_expression();
    }

    fn choose_operator(&mut self, next_operator: Operator) {
        self.clear_error();

        if self.operator.is_some() && !self.waiting_for_second_value {
            self.calculate_result();
        }

        self.first_value = self.current_number();
        self.operator = Some(next_operator);
        self.waiting_for_second_value = true;
        self.update_expression();
    }

    fn calculate_result(&mut self) {
        let Some(first) = self.first_value else {
            return;
        };

        let Some(operator) = self.operator else {
            return;
        };

        if self.waiting_for_second_value {
            return;
        }

        let Some(second) = self.current_number() else {
            self.show_error("Ungueltige Zahl");
            return;
        };

        self.expression = format!("{first} {} {second}", operator.symbol());

        match calculate(first, operator, second) {
            Ok(result) => {
                self.display = format_number(result);
                self.first_value = None;
                self.operator = None;
                self.waiting_for_second_value = false;
                self.has_error = false;
            }
            Err(CalculationError::DivisionByZero) => self.show_error("Division durch 0"),
        }
    }

    fn backspace(&mut self) {
        self.clear_error();

        if self.waiting_for_second_value {
            return;
        }

        self.display.pop();

        if self.display.is_empty() || self.display == "-" {
            self.display = "0".to_owned();
        }

        self.update_expression();
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn current_number(&self) -> Option<f64> {
        self.display.parse::<f64>().ok()
    }

    fn clear_error(&mut self) {
        if self.has_error {
            self.display = "0".to_owned();
            self.expression.clear();
            self.first_value = None;
            self.operator = None;
            self.waiting_for_second_value = false;
            self.has_error = false;
        }
    }

    fn show_error(&mut self, message: &str) {
        self.display = message.to_owned();
        self.first_value = None;
        self.operator = None;
        self.waiting_for_second_value = false;
        self.has_error = true;
    }

    fn update_expression(&mut self) {
        match (self.first_value, self.operator) {
            (Some(first), Some(operator)) if self.waiting_for_second_value => {
                self.expression = format!("{first} {}", operator.symbol());
            }
            (Some(first), Some(operator)) => {
                self.expression = format!("{first} {} {}", operator.symbol(), self.display);
            }
            _ => self.expression.clear(),
        }
    }
}

#[derive(Clone, Copy)]
enum Key {
    Number(char),
    Decimal,
    Operator(Operator),
    Equals,
    Backspace,
    Clear,
}

fn button_for(key: Key, active: bool) -> Button<'static> {
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
            operator.symbol().to_string(),
            if active {
                Color32::from_rgb(244, 248, 251)
            } else {
                Color32::from_rgb(238, 135, 65)
            },
            if active {
                Color32::from_rgb(28, 36, 45)
            } else {
                Color32::from_rgb(255, 255, 255)
            },
        ),
        Key::Equals => (
            "=".to_owned(),
            Color32::from_rgb(42, 146, 103),
            Color32::from_rgb(255, 255, 255),
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
    };

    Button::new(RichText::new(label).color(text_color).size(20.0).strong())
        .fill(color)
        .stroke(egui::Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 24),
        ))
        .corner_radius(10.0)
}

fn format_number(number: f64) -> String {
    let formatted = number.to_string();

    if formatted.ends_with(".0") {
        formatted.trim_end_matches(".0").to_owned()
    } else {
        formatted
    }
}

fn key_color() -> Color32 {
    Color32::from_rgb(49, 61, 74)
}

fn display_expression(expression: &str) -> &str {
    if expression.is_empty() {
        " "
    } else {
        expression
    }
}

fn display_size(display: &str) -> f32 {
    if display.len() > 15 {
        25.0
    } else if display.len() > 10 {
        29.0
    } else {
        36.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press_keys(app: &mut CalculatorApp, keys: &[Key]) {
        for key in keys {
            app.handle_key(*key);
        }
    }

    #[test]
    fn starts_with_zero_display_and_no_pending_operation() {
        let app = CalculatorApp::default();

        assert_eq!(app.display, "0");
        assert_eq!(app.expression, "");
        assert_eq!(app.first_value, None);
        assert_eq!(app.operator, None);
        assert!(!app.waiting_for_second_value);
        assert!(!app.has_error);
    }

    #[test]
    fn appends_digits_and_replaces_initial_zero() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[Key::Number('0'), Key::Number('4'), Key::Number('2')],
        );

        assert_eq!(app.display, "42");
        assert_eq!(app.expression, "");
    }

    #[test]
    fn appends_decimal_only_once() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('1'),
                Key::Decimal,
                Key::Number('5'),
                Key::Decimal,
                Key::Number('2'),
            ],
        );

        assert_eq!(app.display, "1.52");
    }

    #[test]
    fn operator_stores_first_value_and_waits_for_second_number() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::Operator(Operator::Add),
            ],
        );

        assert_eq!(app.first_value, Some(12.0));
        assert_eq!(app.operator, Some(Operator::Add));
        assert_eq!(app.expression, "12 +");
        assert!(app.waiting_for_second_value);
        assert!(app.is_active_operator(Key::Operator(Operator::Add)));
        assert!(!app.is_active_operator(Key::Operator(Operator::Subtract)));
    }

    #[test]
    fn entering_second_number_updates_expression() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::Operator(Operator::Add),
                Key::Number('3'),
            ],
        );

        assert_eq!(app.display, "3");
        assert_eq!(app.expression, "12 + 3");
        assert!(!app.waiting_for_second_value);
    }

    #[test]
    fn calculates_addition() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Add),
                Key::Number('4'),
                Key::Equals,
            ],
        );

        assert_eq!(app.display, "12");
        assert_eq!(app.expression, "8 + 4");
        assert_eq!(app.first_value, None);
        assert_eq!(app.operator, None);
        assert!(!app.has_error);
    }

    #[test]
    fn calculates_subtraction() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Subtract),
                Key::Number('4'),
                Key::Equals,
            ],
        );

        assert_eq!(app.display, "4");
        assert_eq!(app.expression, "8 - 4");
    }

    #[test]
    fn calculates_multiplication() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Multiply),
                Key::Number('4'),
                Key::Equals,
            ],
        );

        assert_eq!(app.display, "32");
        assert_eq!(app.expression, "8 * 4");
    }

    #[test]
    fn calculates_division() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Divide),
                Key::Number('4'),
                Key::Equals,
            ],
        );

        assert_eq!(app.display, "2");
        assert_eq!(app.expression, "8 / 4");
    }

    #[test]
    fn shows_error_for_division_by_zero() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Divide),
                Key::Number('0'),
                Key::Equals,
            ],
        );

        assert_eq!(app.display, "Division durch 0");
        assert_eq!(app.expression, "8 / 0");
        assert!(app.has_error);
        assert_eq!(app.first_value, None);
        assert_eq!(app.operator, None);
    }

    #[test]
    fn next_digit_after_error_resets_state() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Divide),
                Key::Number('0'),
                Key::Equals,
                Key::Number('7'),
            ],
        );

        assert_eq!(app.display, "7");
        assert_eq!(app.expression, "");
        assert!(!app.has_error);
        assert_eq!(app.first_value, None);
        assert_eq!(app.operator, None);
    }

    #[test]
    fn clear_resets_complete_state() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('9'),
                Key::Operator(Operator::Multiply),
                Key::Number('9'),
                Key::Clear,
            ],
        );

        assert_eq!(app.display, "0");
        assert_eq!(app.expression, "");
        assert_eq!(app.first_value, None);
        assert_eq!(app.operator, None);
        assert!(!app.waiting_for_second_value);
        assert!(!app.has_error);
    }

    #[test]
    fn backspace_removes_last_digit_and_falls_back_to_zero() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::Backspace,
                Key::Backspace,
            ],
        );

        assert_eq!(app.display, "0");
    }

    #[test]
    fn backspace_does_nothing_while_waiting_for_second_value() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Subtract),
                Key::Backspace,
            ],
        );

        assert_eq!(app.display, "8");
        assert_eq!(app.expression, "8 -");
        assert!(app.waiting_for_second_value);
    }

    #[test]
    fn operator_chains_pending_calculation() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Add),
                Key::Number('4'),
                Key::Operator(Operator::Multiply),
            ],
        );

        assert_eq!(app.display, "12");
        assert_eq!(app.first_value, Some(12.0));
        assert_eq!(app.operator, Some(Operator::Multiply));
        assert_eq!(app.expression, "12 *");
        assert!(app.waiting_for_second_value);
    }

    #[test]
    fn equals_without_complete_operation_is_noop() {
        let mut app = CalculatorApp::default();

        press_keys(&mut app, &[Key::Number('5'), Key::Equals]);

        assert_eq!(app.display, "5");
        assert_eq!(app.expression, "");
        assert_eq!(app.first_value, None);
        assert_eq!(app.operator, None);
    }

    #[test]
    fn decimal_after_operator_starts_second_number() {
        let mut app = CalculatorApp::default();

        press_keys(
            &mut app,
            &[
                Key::Number('5'),
                Key::Operator(Operator::Add),
                Key::Decimal,
                Key::Number('2'),
            ],
        );

        assert_eq!(app.display, "0.2");
        assert_eq!(app.expression, "5 + 0.2");
    }

    #[test]
    fn formats_integer_results_without_decimal_suffix() {
        assert_eq!(format_number(12.0), "12");
    }

    #[test]
    fn keeps_fractional_results() {
        assert_eq!(format_number(2.5), "2.5");
    }

    #[test]
    fn empty_expression_renders_as_blank_space() {
        assert_eq!(display_expression(""), " ");
        assert_eq!(display_expression("8 + 4"), "8 + 4");
    }

    #[test]
    fn display_size_shrinks_for_long_values() {
        assert_eq!(display_size("1234567890"), 36.0);
        assert_eq!(display_size("12345678901"), 29.0);
        assert_eq!(display_size("1234567890123456"), 25.0);
    }
}
