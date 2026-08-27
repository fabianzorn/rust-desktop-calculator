//! Calculator interaction state independent of egui rendering.

use crate::calculator::{
    AngleMode, CalculationError, MathematicalConstant, Operator, UnaryOperator, calculate,
    calculate_unary,
};

use super::formatting::{format_number, format_unary_expression};
use super::input::Key;

const MAX_INPUT_LENGTH: usize = 18;

/// Arithmetic state suspended while a parenthesized expression is evaluated.
struct ParenthesisContext {
    first_value: Option<f64>,
    operator: Option<Operator>,
    prefix: String,
}

/// Owns the displayed values and any pending arithmetic operation.
pub(super) struct CalculatorState {
    first_value: Option<f64>,
    operator: Option<Operator>,
    display: String,
    expression: String,
    waiting_for_second_value: bool,
    has_error: bool,
    angle_mode: AngleMode,
    memory: f64,
    parentheses: Vec<ParenthesisContext>,
    value_entered: bool,
    grouped_operand: bool,
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
            memory: 0.0,
            parentheses: Vec::new(),
            value_entered: false,
            grouped_operand: false,
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

    /// Reports whether calculator memory currently contains a non-zero value.
    pub(super) fn has_memory(&self) -> bool {
        self.memory != 0.0
    }

    /// Reports whether a parenthesized expression is currently incomplete.
    pub(super) fn has_open_parentheses(&self) -> bool {
        !self.parentheses.is_empty()
    }

    /// Applies one button or keyboard action to the calculator state.
    pub(super) fn handle_key(&mut self, key: Key) {
        match key {
            Key::Number(number) => self.append_number(number),
            Key::Decimal => self.append_decimal(),
            Key::Operator(operator) => self.choose_operator(operator),
            Key::UnaryOperator(operator) => self.apply_unary_operator(operator),
            Key::Constant(constant) => self.insert_constant(constant),
            Key::MemoryClear => self.clear_memory(),
            Key::MemoryRecall => self.recall_memory(),
            Key::MemoryAdd => self.update_memory(1.0),
            Key::MemorySubtract => self.update_memory(-1.0),
            Key::OpenParenthesis => self.open_parenthesis(),
            Key::CloseParenthesis => self.close_parenthesis(),
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
        } else if self.display.len() < MAX_INPUT_LENGTH {
            self.display.push(number);
        }

        self.value_entered = true;
        self.grouped_operand = false;
        self.update_expression();
    }

    fn append_decimal(&mut self) {
        self.clear_error();

        if self.waiting_for_second_value {
            self.display = "0.".to_owned();
            self.waiting_for_second_value = false;
        } else if !self.display.contains('.') && self.display.len() < MAX_INPUT_LENGTH {
            self.display.push('.');
        }

        self.value_entered = true;
        self.grouped_operand = false;
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
        self.grouped_operand = false;
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

        if !self.grouped_operand {
            self.update_expression();
        }

        match calculate(first, operator, second) {
            Ok(result) => {
                self.display = format_number(result);
                self.first_value = None;
                self.operator = None;
                self.waiting_for_second_value = false;
                self.has_error = false;
                self.value_entered = true;
                self.grouped_operand = false;
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
                self.value_entered = true;
                self.grouped_operand = false;

                if self.first_value.is_some() && self.operator.is_some() {
                    self.update_expression();
                } else {
                    self.set_standalone_expression(unary_expression);
                }
            }
            Err(error) => {
                self.set_standalone_expression(unary_expression);
                self.show_calculation_error(error);
            }
        }
    }

    fn insert_constant(&mut self, constant: MathematicalConstant) {
        self.clear_error();
        self.display = format_number(constant.value());
        self.value_entered = true;
        self.grouped_operand = false;

        if self.waiting_for_second_value {
            self.waiting_for_second_value = false;
            self.update_expression();
        } else if self.first_value.is_some() && self.operator.is_some() {
            self.update_expression();
        } else {
            self.set_standalone_expression(constant.symbol().to_owned());
        }
    }

    fn clear_memory(&mut self) {
        if !self.has_error {
            self.memory = 0.0;
        }
    }

    fn recall_memory(&mut self) {
        if self.has_error {
            return;
        }

        self.display = format_number(self.memory);
        self.value_entered = true;
        self.grouped_operand = false;
        if self.waiting_for_second_value {
            self.waiting_for_second_value = false;
            self.update_expression();
        } else if self.first_value.is_some() && self.operator.is_some() {
            self.update_expression();
        } else {
            self.set_standalone_expression("MR".to_owned());
        }
    }

    fn update_memory(&mut self, direction: f64) {
        if self.has_error {
            return;
        }
        let Some(value) = self.current_number() else {
            return;
        };

        let updated = self.memory + direction * value;
        if updated.is_finite() {
            self.memory = updated;
        }
    }

    fn set_standalone_expression(&mut self, expression: String) {
        if let Some(context) = self.parentheses.last() {
            self.expression = format!("{}{expression}", context.prefix);
        } else {
            self.expression = expression;
        }
    }

    fn open_parenthesis(&mut self) {
        self.clear_error();

        if self.value_entered && !self.waiting_for_second_value {
            return;
        }

        let prefix = if self.expression.is_empty() {
            "(".to_owned()
        } else {
            format!("{} (", self.expression)
        };
        self.parentheses.push(ParenthesisContext {
            first_value: self.first_value,
            operator: self.operator,
            prefix: prefix.clone(),
        });
        self.first_value = None;
        self.operator = None;
        self.display = "0".to_owned();
        self.expression = prefix;
        self.waiting_for_second_value = false;
        self.value_entered = false;
        self.grouped_operand = false;
    }

    fn close_parenthesis(&mut self) {
        if self.has_error
            || self.parentheses.is_empty()
            || !self.value_entered
            || self.waiting_for_second_value
        {
            return;
        }

        if self.operator.is_some() {
            self.calculate_result();
            if self.has_error {
                return;
            }
        }

        let result = self.current_number().unwrap_or_default();
        let context = self.parentheses.pop().expect("parenthesis checked above");
        let inner_expression = self
            .expression
            .strip_prefix(&context.prefix)
            .unwrap_or(self.display())
            .to_owned();
        self.first_value = context.first_value;
        self.operator = context.operator;
        self.display = format_number(result);
        self.expression = format!("{}{inner_expression})", context.prefix);
        self.waiting_for_second_value = false;
        self.value_entered = true;
        self.grouped_operand = true;
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
        let memory = self.memory;
        *self = Self {
            angle_mode,
            memory,
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
        let local_expression = match (self.first_value, self.operator) {
            (Some(first), Some(operator)) if self.waiting_for_second_value => {
                format_binary_expression(first, operator, None)
            }
            (Some(first), Some(operator)) => {
                format_binary_expression(first, operator, Some(&self.display))
            }
            _ if self.value_entered && !self.parentheses.is_empty() => self.display.clone(),
            _ => String::new(),
        };

        if let Some(context) = self.parentheses.last() {
            self.expression = format!("{}{local_expression}", context.prefix);
        } else {
            self.expression = local_expression;
        }
    }
}

/// Formats a pending or complete binary expression for the secondary display.
fn format_binary_expression(first: f64, operator: Operator, second: Option<&str>) -> String {
    match (operator, second) {
        (Operator::ScientificNotation, Some(second)) => format!("{first} × 10^{second}"),
        (Operator::ScientificNotation, None) => format!("{first} × 10^"),
        (_, Some(second)) => format!("{first} {} {second}", operator.symbol()),
        (_, None) => format!("{first} {}", operator.symbol()),
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
        assert_eq!(state.memory, 0.0);
        assert!(!state.has_memory());
        assert!(state.parentheses.is_empty());
        assert!(!state.value_entered);
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
    fn limits_manually_entered_numbers() {
        let mut state = CalculatorState::default();
        enter_number(&mut state, "12345678901234567890");

        assert_eq!(state.display, "123456789012345678");
        assert_eq!(state.display.len(), MAX_INPUT_LENGTH);
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
    fn calculates_modulo_and_scientific_notation() {
        let mut modulo = CalculatorState::default();
        press_keys(
            &mut modulo,
            &[
                Key::Number('1'),
                Key::Number('7'),
                Key::Operator(Operator::Modulo),
                Key::Number('5'),
                Key::Equals,
            ],
        );
        assert_eq!(modulo.display, "2");
        assert_eq!(modulo.expression, "17 mod 5");

        let mut scientific = CalculatorState::default();
        press_keys(
            &mut scientific,
            &[
                Key::Number('2'),
                Key::Operator(Operator::ScientificNotation),
                Key::Number('3'),
                Key::Equals,
            ],
        );
        assert_eq!(scientific.display, "2000");
        assert_eq!(scientific.expression, "2 × 10^3");
    }

    #[test]
    fn calculates_parenthesized_expression() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('2'),
                Key::Operator(Operator::Multiply),
                Key::OpenParenthesis,
                Key::Number('3'),
                Key::Operator(Operator::Add),
                Key::Number('4'),
                Key::CloseParenthesis,
                Key::Equals,
            ],
        );

        assert_eq!(state.display, "14");
        assert_eq!(state.expression, "2 * (3 + 4)");
        assert!(state.parentheses.is_empty());
    }

    #[test]
    fn calculates_nested_parentheses() {
        let mut state = CalculatorState::default();
        press_keys(
            &mut state,
            &[
                Key::Number('2'),
                Key::Operator(Operator::Add),
                Key::OpenParenthesis,
                Key::Number('3'),
                Key::Operator(Operator::Multiply),
                Key::OpenParenthesis,
                Key::Number('4'),
                Key::Operator(Operator::Add),
                Key::Number('1'),
                Key::CloseParenthesis,
                Key::CloseParenthesis,
                Key::Equals,
            ],
        );

        assert_eq!(state.display, "17");
        assert_eq!(state.expression, "2 + (3 * (4 + 1))");
        assert!(state.parentheses.is_empty());
    }

    #[test]
    fn ignores_unmatched_or_empty_parentheses() {
        let mut state = CalculatorState::default();
        state.handle_key(Key::CloseParenthesis);
        assert_eq!(state.display, "0");
        assert_eq!(state.expression, "");

        state.handle_key(Key::OpenParenthesis);
        state.handle_key(Key::CloseParenthesis);
        assert_eq!(state.display, "0");
        assert_eq!(state.expression, "(");
        assert_eq!(state.parentheses.len(), 1);

        state.handle_key(Key::Clear);
        assert!(state.parentheses.is_empty());
        assert_eq!(state.expression, "");
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
    fn calculates_reference_image_functions() {
        let cases = [
            ("0", UnaryOperator::HyperbolicSine, "0", "sinh(0)"),
            ("0", UnaryOperator::HyperbolicCosine, "1", "cosh(0)"),
            ("0", UnaryOperator::HyperbolicTangent, "0", "tanh(0)"),
            ("2.9", UnaryOperator::Floor, "2", "floor(2.9)"),
            ("2.1", UnaryOperator::Ceiling, "3", "ceil(2.1)"),
        ];

        for (input, operator, result, expression) in cases {
            let mut state = CalculatorState::default();
            enter_number(&mut state, input);
            state.handle_key(Key::UnaryOperator(operator));

            assert_eq!(state.display, result);
            assert_eq!(state.expression, expression);
            assert!(!state.has_error);
        }

        let mut absolute = CalculatorState::default();
        enter_number(&mut absolute, "2.5");
        press_keys(
            &mut absolute,
            &[
                Key::UnaryOperator(UnaryOperator::ToggleSign),
                Key::UnaryOperator(UnaryOperator::AbsoluteValue),
            ],
        );
        assert_eq!(absolute.display, "2.5");
        assert_eq!(absolute.expression, "|-2.5|");
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
    fn adds_to_and_subtracts_from_memory() {
        let mut state = CalculatorState::default();
        enter_number(&mut state, "10");
        state.handle_key(Key::MemoryAdd);
        state.handle_key(Key::Clear);
        enter_number(&mut state, "3");
        state.handle_key(Key::MemorySubtract);

        assert_eq!(state.memory, 7.0);
        assert!(state.has_memory());

        state.handle_key(Key::Clear);
        state.handle_key(Key::MemoryRecall);
        assert_eq!(state.display, "7");
        assert_eq!(state.expression, "MR");
    }

    #[test]
    fn memory_survives_clear_and_is_removed_by_memory_clear() {
        let mut state = CalculatorState::default();
        enter_number(&mut state, "5");
        state.handle_key(Key::MemoryAdd);
        state.handle_key(Key::Clear);

        assert_eq!(state.memory, 5.0);
        assert!(state.has_memory());

        state.handle_key(Key::MemoryClear);
        assert_eq!(state.memory, 0.0);
        assert!(!state.has_memory());
    }

    #[test]
    fn recalls_memory_as_second_operand() {
        let mut state = CalculatorState::default();
        enter_number(&mut state, "4");
        state.handle_key(Key::MemoryAdd);
        state.handle_key(Key::Clear);
        press_keys(
            &mut state,
            &[
                Key::Number('2'),
                Key::Operator(Operator::Add),
                Key::MemoryRecall,
                Key::Equals,
            ],
        );

        assert_eq!(state.display, "6");
        assert_eq!(state.expression, "2 + 4");
    }

    #[test]
    fn memory_operations_are_noops_during_errors() {
        let mut state = CalculatorState::default();
        enter_number(&mut state, "5");
        state.handle_key(Key::MemoryAdd);
        state.handle_key(Key::Clear);
        press_keys(
            &mut state,
            &[
                Key::Number('8'),
                Key::Operator(Operator::Divide),
                Key::Number('0'),
                Key::Equals,
                Key::MemoryAdd,
                Key::MemorySubtract,
                Key::MemoryRecall,
                Key::MemoryClear,
            ],
        );

        assert_eq!(state.display, "Cannot divide by zero");
        assert_eq!(state.memory, 5.0);
        assert!(state.has_memory());
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
