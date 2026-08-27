//! Calculator interaction state independent of egui rendering.

use crate::calculator::{
    AngleMode, CalculationError, MathematicalConstant, Operator, UnaryOperator, calculate,
    calculate_unary,
};

use super::formatting::{format_number, format_unary_expression};
use super::input::Key;

/// Owns the displayed values and any pending arithmetic operation.
pub(super) struct CalculatorState {
    first_value: Option<f64>,
    operator: Option<Operator>,
    display: String,
    expression: String,
    waiting_for_second_value: bool,
    has_error: bool,
    angle_mode: AngleMode,
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self {
            first_value: None,
            operator: None,
            display: "0".to_owned(),
            expression: String::new(),
            waiting_for_second_value: false,
            has_error: false,
            angle_mode: AngleMode::default(),
        }
    }
}

impl CalculatorState {
    /// Returns the value or error message shown in the main display.
    pub(super) fn display(&self) -> &str {
        &self.display
    }

    /// Returns the arithmetic expression shown above the main display.
    pub(super) fn expression(&self) -> &str {
        &self.expression
    }

    /// Reports whether the main display currently contains an error.
    pub(super) fn has_error(&self) -> bool {
        self.has_error
    }

    /// Returns the angle unit used by trigonometric operations.
    pub(super) fn angle_mode(&self) -> AngleMode {
        self.angle_mode
    }

    /// Applies one button or keyboard action to the calculator state.
    pub(super) fn handle_key(&mut self, key: Key) {
        match key {
            Key::Number(number) => self.append_number(number),
            Key::Decimal => self.append_decimal(),
            Key::Operator(operator) => self.choose_operator(operator),
            Key::UnaryOperator(operator) => self.apply_unary_operator(operator),
            Key::Constant(constant) => self.insert_constant(constant),
            Key::Equals => self.calculate_result(),
            Key::Backspace => self.backspace(),
            Key::Clear => self.clear(),
            Key::ToggleAngleMode => self.toggle_angle_mode(),
        }
    }

    /// Reports whether `key` represents the pending binary operator.
    pub(super) fn is_active_operator(&self, key: Key) -> bool {
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
            self.show_error("Invalid number");
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
            Err(error) => self.show_calculation_error(error),
        }
    }

    fn apply_unary_operator(&mut self, operator: UnaryOperator) {
        self.clear_error();

        if self.waiting_for_second_value {
            return;
        }
        let Some(value) = self.current_number() else {
            self.show_error("Invalid number");
            return;
        };

        let unary_expression = format_unary_expression(value, operator, self.angle_mode);

        match calculate_unary(value, operator, self.angle_mode) {
            Ok(result) => {
                self.display = format_number(result);
                self.has_error = false;

                if self.first_value.is_some() && self.operator.is_some() {
                    self.update_expression();
                } else {
                    self.expression = unary_expression;
                }
            }
            Err(error) => {
                self.expression = unary_expression;
                self.show_calculation_error(error);
            }
        }
    }

    fn insert_constant(&mut self, constant: MathematicalConstant) {
        self.clear_error();
        self.display = format_number(constant.value());

        if self.waiting_for_second_value {
            self.waiting_for_second_value = false;
            self.update_expression();
        } else if self.first_value.is_some() && self.operator.is_some() {
            self.update_expression();
        } else {
            self.expression = constant.symbol().to_owned();
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
        self.reset_calculation();
    }

    /// Switches the angle unit without changing the current calculation.
    fn toggle_angle_mode(&mut self) {
        self.angle_mode = self.angle_mode.toggled();
    }

    fn current_number(&self) -> Option<f64> {
        self.display.parse::<f64>().ok()
    }

    fn clear_error(&mut self) {
        if self.has_error {
            self.reset_calculation();
        }
    }

    /// Resets calculation data while retaining the selected angle mode.
    fn reset_calculation(&mut self) {
        let angle_mode = self.angle_mode;
        *self = Self {
            angle_mode,
            ..Self::default()
        };
    }

    fn show_error(&mut self, message: &str) {
        self.display = message.to_owned();
        self.first_value = None;
        self.operator = None;
        self.waiting_for_second_value = false;
        self.has_error = true;
    }

    fn show_calculation_error(&mut self, error: CalculationError) {
        match error {
            CalculationError::DivisionByZero => self.show_error("Cannot divide by zero"),
            CalculationError::NegativeSquareRoot => self.show_error("Invalid square root"),
            CalculationError::UndefinedTangent => self.show_error("Undefined tangent"),
            CalculationError::InvalidLogarithm => self.show_error("Invalid logarithm"),
            CalculationError::InvalidPower => self.show_error("Invalid power"),
            CalculationError::InvalidFactorial => self.show_error("Invalid factorial"),
            CalculationError::ResultOutOfRange => self.show_error("Result out of range"),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn press_keys(state: &mut CalculatorState, keys: &[Key]) {
        for key in keys {
            state.handle_key(*key);
        }
    }

    fn enter_number(state: &mut CalculatorState, number: &str) {
        for character in number.chars() {
            let key = match character {
                '.' => Key::Decimal,
                digit => Key::Number(digit),
            };
            state.handle_key(key);
        }
    }

    #[test]
    fn starts_with_zero_display_and_no_pending_operation() {
        let state = CalculatorState::default();

        assert_eq!(state.display, "0");
        assert_eq!(state.expression, "");
        assert_eq!(state.first_value, None);
        assert_eq!(state.operator, None);
        assert!(!state.waiting_for_second_value);
        assert!(!state.has_error);
        assert_eq!(state.angle_mode, AngleMode::Degrees);
    }

    #[test]
    fn appends_digits_and_replaces_initial_zero() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[Key::Number('0'), Key::Number('4'), Key::Number('2')],
        );
        assert_eq!(state.display, "42");
        assert_eq!(state.expression, "");
    }

    #[test]
    fn appends_decimal_only_once() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('1'),
                Key::Decimal,
                Key::Number('5'),
                Key::Decimal,
                Key::Number('2'),
            ],
        );
        assert_eq!(state.display, "1.52");
    }

    #[test]
    fn operator_stores_first_value_and_waits_for_second_number() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::Operator(Operator::Add),
            ],
        );

        assert_eq!(state.first_value, Some(12.0));
        assert_eq!(state.operator, Some(Operator::Add));
        assert_eq!(state.expression, "12 +");
        assert!(state.waiting_for_second_value);
        assert!(state.is_active_operator(Key::Operator(Operator::Add)));
        assert!(!state.is_active_operator(Key::Operator(Operator::Subtract)));
    }

    #[test]
    fn entering_second_number_updates_expression() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::Operator(Operator::Add),
                Key::Number('3'),
            ],
        );
        assert_eq!(state.display, "3");
        assert_eq!(state.expression, "12 + 3");
        assert!(!state.waiting_for_second_value);
    }

    #[test]
    fn calculates_addition() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Add),
                Key::Number('4'),
                Key::Equals,
            ],
        );
        assert_eq!(state.display, "12");
        assert_eq!(state.expression, "8 + 4");
        assert_eq!(state.first_value, None);
        assert_eq!(state.operator, None);
        assert!(!state.has_error);
    }

    #[test]
    fn calculates_subtraction() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Subtract),
                Key::Number('4'),
                Key::Equals,
            ],
        );
        assert_eq!(state.display, "4");
        assert_eq!(state.expression, "8 - 4");
    }

    #[test]
    fn calculates_multiplication() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Multiply),
                Key::Number('4'),
                Key::Equals,
            ],
        );
        assert_eq!(state.display, "32");
        assert_eq!(state.expression, "8 * 4");
    }

    #[test]
    fn calculates_division() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Divide),
                Key::Number('4'),
                Key::Equals,
            ],
        );
        assert_eq!(state.display, "2");
        assert_eq!(state.expression, "8 / 4");
    }

    #[test]
    fn calculates_power() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('2'),
                Key::Operator(Operator::Power),
                Key::Number('8'),
                Key::Equals,
            ],
        );
        assert_eq!(state.display, "256");
        assert_eq!(state.expression, "2 ^ 8");
    }

    #[test]
    fn shows_error_for_division_by_zero() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Divide),
                Key::Number('0'),
                Key::Equals,
            ],
        );
        assert_eq!(state.display, "Cannot divide by zero");
        assert_eq!(state.expression, "8 / 0");
        assert!(state.has_error);
        assert_eq!(state.first_value, None);
        assert_eq!(state.operator, None);
    }

    #[test]
    fn next_digit_after_error_resets_state() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Divide),
                Key::Number('0'),
                Key::Equals,
                Key::Number('7'),
            ],
        );
        assert_eq!(state.display, "7");
        assert_eq!(state.expression, "");
        assert!(!state.has_error);
        assert_eq!(state.first_value, None);
        assert_eq!(state.operator, None);
    }

    #[test]
    fn clear_resets_complete_state() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('9'),
                Key::Operator(Operator::Multiply),
                Key::Number('9'),
                Key::Clear,
            ],
        );
        assert_eq!(state.display, "0");
        assert_eq!(state.expression, "");
        assert_eq!(state.first_value, None);
        assert_eq!(state.operator, None);
        assert!(!state.waiting_for_second_value);
        assert!(!state.has_error);
    }

    #[test]
    fn backspace_removes_last_digit_and_falls_back_to_zero() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::Backspace,
                Key::Backspace,
            ],
        );
        assert_eq!(state.display, "0");
    }

    #[test]
    fn backspace_does_nothing_while_waiting_for_second_value() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Subtract),
                Key::Backspace,
            ],
        );
        assert_eq!(state.display, "8");
        assert_eq!(state.expression, "8 -");
        assert!(state.waiting_for_second_value);
    }

    #[test]
    fn operator_chains_pending_calculation() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Add),
                Key::Number('4'),
                Key::Operator(Operator::Multiply),
            ],
        );
        assert_eq!(state.display, "12");
        assert_eq!(state.first_value, Some(12.0));
        assert_eq!(state.operator, Some(Operator::Multiply));
        assert_eq!(state.expression, "12 *");
    }

    #[test]
    fn equals_without_complete_operation_is_noop() {
        let mut state = CalculatorState::default();
        press_keys(&mut state, &[Key::Number('5'), Key::Equals]);
        assert_eq!(state.display, "5");
        assert_eq!(state.expression, "");
    }

    #[test]
    fn decimal_after_operator_starts_second_number() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('5'),
                Key::Operator(Operator::Add),
                Key::Decimal,
                Key::Number('2'),
            ],
        );
        assert_eq!(state.display, "0.2");
        assert_eq!(state.expression, "5 + 0.2");
    }

    #[test]
    fn toggles_the_sign_of_the_displayed_number() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('4'),
                Key::UnaryOperator(UnaryOperator::ToggleSign),
            ],
        );
        assert_eq!(state.display, "-4");
        assert_eq!(state.expression, "-(4)");

        state.handle_key(Key::UnaryOperator(UnaryOperator::ToggleSign));
        assert_eq!(state.display, "4");
        assert_eq!(state.expression, "-(-4)");
    }

    #[test]
    fn converts_number_to_percentage() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('2'),
                Key::Number('5'),
                Key::UnaryOperator(UnaryOperator::Percent),
            ],
        );
        assert_eq!(state.display, "0.25");
        assert_eq!(state.expression, "25%");
    }

    #[test]
    fn calculates_square_root() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Number('1'),
                Key::UnaryOperator(UnaryOperator::SquareRoot),
            ],
        );
        assert_eq!(state.display, "9");
        assert_eq!(state.expression, "√(81)");
        assert!(!state.has_error);
    }

    #[test]
    fn calculates_square() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::UnaryOperator(UnaryOperator::Square),
            ],
        );
        assert_eq!(state.display, "144");
        assert_eq!(state.expression, "(12)²");
    }

    #[test]
    fn calculates_additional_scientific_functions() {
        let cases = [
            ("100", UnaryOperator::LogarithmBase10, "2", "log₁₀(100)"),
            ("1", UnaryOperator::NaturalLogarithm, "0", "ln(1)"),
            ("0", UnaryOperator::Exponential, "1", "e^(0)"),
            ("4", UnaryOperator::Reciprocal, "0.25", "1/(4)"),
            ("5", UnaryOperator::Factorial, "120", "(5)!"),
        ];

        for (input, operator, result, expression) in cases {
            let mut state = CalculatorState::default();
            enter_number(&mut state, input);
            state.handle_key(Key::UnaryOperator(operator));

            assert_eq!(state.display, result);
            assert_eq!(state.expression, expression);
            assert!(!state.has_error);
        }
    }

    #[test]
    fn inserts_mathematical_constants() {
        let mut state = CalculatorState::default();
        state.handle_key(Key::Constant(MathematicalConstant::Pi));
        assert_eq!(state.display, std::f64::consts::PI.to_string());
        assert_eq!(state.expression, "π");

        state.handle_key(Key::Constant(MathematicalConstant::Euler));
        assert_eq!(state.display, std::f64::consts::E.to_string());
        assert_eq!(state.expression, "e");
    }

    #[test]
    fn uses_constant_as_second_operand() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('2'),
                Key::Operator(Operator::Multiply),
                Key::Constant(MathematicalConstant::Pi),
                Key::Equals,
            ],
        );

        assert_eq!(state.display, (2.0 * std::f64::consts::PI).to_string());
        assert_eq!(state.expression, format!("2 * {}", std::f64::consts::PI));
    }

    #[test]
    fn shows_errors_for_invalid_scientific_function_domains() {
        let cases = [
            (
                "0",
                UnaryOperator::NaturalLogarithm,
                "Invalid logarithm",
                "ln(0)",
            ),
            (
                "0",
                UnaryOperator::Reciprocal,
                "Cannot divide by zero",
                "1/(0)",
            ),
            (
                "1.5",
                UnaryOperator::Factorial,
                "Invalid factorial",
                "(1.5)!",
            ),
        ];

        for (input, operator, message, expression) in cases {
            let mut state = CalculatorState::default();
            enter_number(&mut state, input);
            state.handle_key(Key::UnaryOperator(operator));

            assert_eq!(state.display, message);
            assert_eq!(state.expression, expression);
            assert!(state.has_error);
        }
    }

    #[test]
    fn calculates_sine_in_degrees() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('3'),
                Key::Number('0'),
                Key::UnaryOperator(UnaryOperator::Sine),
            ],
        );
        assert_eq!(state.display, "0.5");
        assert_eq!(state.expression, "sin(30°)");
    }

    #[test]
    fn calculates_cosine_in_degrees() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('6'),
                Key::Number('0'),
                Key::UnaryOperator(UnaryOperator::Cosine),
            ],
        );
        assert_eq!(state.display, "0.5");
        assert_eq!(state.expression, "cos(60°)");
    }

    #[test]
    fn calculates_tangent_in_degrees() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('4'),
                Key::Number('5'),
                Key::UnaryOperator(UnaryOperator::Tangent),
            ],
        );
        assert_eq!(state.display, "1");
        assert_eq!(state.expression, "tan(45°)");
    }

    #[test]
    fn calculates_trigonometry_in_radians() {
        let mut state = CalculatorState::default();
        state.handle_key(Key::ToggleAngleMode);
        enter_number(&mut state, "1.5707963267948966");
        state.handle_key(Key::UnaryOperator(UnaryOperator::Sine));

        assert_eq!(state.angle_mode, AngleMode::Radians);
        assert_eq!(state.display, "1");
        assert_eq!(state.expression, "sin(1.5707963267948966 rad)");
    }

    #[test]
    fn angle_mode_survives_clear() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::ToggleAngleMode,
                Key::Number('4'),
                Key::Number('2'),
                Key::Clear,
            ],
        );

        assert_eq!(state.angle_mode, AngleMode::Radians);
        assert_eq!(state.display, "0");
        assert_eq!(state.expression, "");
    }

    #[test]
    fn shows_error_for_undefined_tangent() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('9'),
                Key::Number('0'),
                Key::UnaryOperator(UnaryOperator::Tangent),
            ],
        );
        assert_eq!(state.display, "Undefined tangent");
        assert_eq!(state.expression, "tan(90°)");
        assert!(state.has_error);
    }

    #[test]
    fn shows_error_for_square_root_of_negative_number() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('4'),
                Key::UnaryOperator(UnaryOperator::ToggleSign),
                Key::UnaryOperator(UnaryOperator::SquareRoot),
            ],
        );
        assert_eq!(state.display, "Invalid square root");
        assert_eq!(state.expression, "√(-4)");
        assert!(state.has_error);
    }

    #[test]
    fn applies_unary_operation_to_second_operand() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('1'),
                Key::Number('0'),
                Key::Operator(Operator::Add),
                Key::Number('2'),
                Key::Number('5'),
                Key::UnaryOperator(UnaryOperator::Percent),
                Key::Equals,
            ],
        );
        assert_eq!(state.display, "10.25");
        assert_eq!(state.expression, "10 + 0.25");
    }

    #[test]
    fn unary_operation_while_waiting_for_operand_is_noop() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Add),
                Key::UnaryOperator(UnaryOperator::Square),
            ],
        );
        assert_eq!(state.display, "8");
        assert_eq!(state.expression, "8 +");
        assert!(state.waiting_for_second_value);
    }
}
