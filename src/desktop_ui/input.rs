//! Translation of keyboard events into calculator actions.

use eframe::egui;

use crate::calculator::{MathematicalConstant, Operator, UnaryOperator};

/// An action that can be triggered by a calculator button or keyboard input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Key {
    /// Enters a decimal digit.
    Number(char),
    /// Enters the decimal separator.
    Decimal,
    /// Selects a binary operation.
    Operator(Operator),
    /// Applies an operation to the currently displayed value.
    UnaryOperator(UnaryOperator),
    /// Enters a mathematical constant.
    Constant(MathematicalConstant),
    /// Clears the calculator memory.
    MemoryClear,
    /// Recalls the value stored in calculator memory.
    MemoryRecall,
    /// Adds the displayed value to calculator memory.
    MemoryAdd,
    /// Subtracts the displayed value from calculator memory.
    MemorySubtract,
    /// Opens a grouped sub-expression.
    OpenParenthesis,
    /// Closes and evaluates the current grouped sub-expression.
    CloseParenthesis,
    /// Evaluates the pending binary operation.
    Equals,
    /// Removes the last digit from the current value.
    Backspace,
    /// Resets the complete calculator state.
    Clear,
    /// Switches between degree and radian angle modes.
    ToggleAngleMode,
}

/// Collects all calculator actions triggered during the current input frame.
pub(super) fn keyboard_keys(context: &egui::Context) -> Vec<Key> {
    context.input(|input| {
        let mut keys = if input.modifiers.command {
            Vec::new()
        } else {
            input
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::Text(text) => text.chars().find_map(key_from_character),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };

        if input.key_pressed(egui::Key::Enter) {
            keys.push(Key::Equals);
        }
        if input.key_pressed(egui::Key::Backspace) {
            keys.push(Key::Backspace);
        }
        if input.key_pressed(egui::Key::Escape) || input.key_pressed(egui::Key::Delete) {
            keys.push(Key::Clear);
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::L) {
            keys.push(Key::MemoryClear);
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::R) {
            keys.push(Key::MemoryRecall);
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::P) {
            keys.push(Key::MemoryAdd);
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::Q) {
            keys.push(Key::MemorySubtract);
        }

        keys
    })
}

/// Reports whether the platform's standard copy shortcut was pressed.
pub(super) fn copy_result_requested(context: &egui::Context) -> bool {
    context.input(|input| input.modifiers.command && input.key_pressed(egui::Key::C))
}

/// Reports whether the calculator-view mode shortcut was pressed.
pub(super) fn calculator_mode_toggle_requested(context: &egui::Context) -> bool {
    context.input(|input| input.key_pressed(egui::Key::F2))
}

/// Maps a typed character to its calculator action.
fn key_from_character(character: char) -> Option<Key> {
    match character {
        '0'..='9' => Some(Key::Number(character)),
        '.' | ',' => Some(Key::Decimal),
        '+' => Some(Key::Operator(Operator::Add)),
        '-' => Some(Key::Operator(Operator::Subtract)),
        '*' | '×' => Some(Key::Operator(Operator::Multiply)),
        '/' | '÷' => Some(Key::Operator(Operator::Divide)),
        '^' => Some(Key::Operator(Operator::Power)),
        '=' => Some(Key::Equals),
        '%' => Some(Key::UnaryOperator(UnaryOperator::Percent)),
        'r' | 'R' => Some(Key::UnaryOperator(UnaryOperator::SquareRoot)),
        's' | 'S' => Some(Key::UnaryOperator(UnaryOperator::Square)),
        'n' | 'N' => Some(Key::UnaryOperator(UnaryOperator::ToggleSign)),
        'i' | 'I' => Some(Key::UnaryOperator(UnaryOperator::Sine)),
        'c' | 'C' => Some(Key::UnaryOperator(UnaryOperator::Cosine)),
        't' | 'T' => Some(Key::UnaryOperator(UnaryOperator::Tangent)),
        'm' | 'M' => Some(Key::ToggleAngleMode),
        'o' | 'O' => Some(Key::UnaryOperator(UnaryOperator::LogarithmBase10)),
        'l' | 'L' => Some(Key::UnaryOperator(UnaryOperator::NaturalLogarithm)),
        'e' | 'E' => Some(Key::UnaryOperator(UnaryOperator::Exponential)),
        'v' | 'V' => Some(Key::UnaryOperator(UnaryOperator::Reciprocal)),
        'f' | 'F' => Some(Key::UnaryOperator(UnaryOperator::Factorial)),
        'p' | 'P' => Some(Key::Constant(MathematicalConstant::Pi)),
        'k' | 'K' => Some(Key::Constant(MathematicalConstant::Euler)),
        '(' => Some(Key::OpenParenthesis),
        ')' => Some(Key::CloseParenthesis),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_keyboard_characters_to_calculator_keys() {
        assert_eq!(key_from_character('7'), Some(Key::Number('7')));
        assert_eq!(key_from_character('+'), Some(Key::Operator(Operator::Add)));
        assert_eq!(
            key_from_character('r'),
            Some(Key::UnaryOperator(UnaryOperator::SquareRoot))
        );
        assert_eq!(
            key_from_character('i'),
            Some(Key::UnaryOperator(UnaryOperator::Sine))
        );
        assert_eq!(
            key_from_character('c'),
            Some(Key::UnaryOperator(UnaryOperator::Cosine))
        );
        assert_eq!(
            key_from_character('t'),
            Some(Key::UnaryOperator(UnaryOperator::Tangent))
        );
        assert_eq!(key_from_character('m'), Some(Key::ToggleAngleMode));
        assert_eq!(
            key_from_character('o'),
            Some(Key::UnaryOperator(UnaryOperator::LogarithmBase10))
        );
        assert_eq!(
            key_from_character('l'),
            Some(Key::UnaryOperator(UnaryOperator::NaturalLogarithm))
        );
        assert_eq!(
            key_from_character('e'),
            Some(Key::UnaryOperator(UnaryOperator::Exponential))
        );
        assert_eq!(
            key_from_character('v'),
            Some(Key::UnaryOperator(UnaryOperator::Reciprocal))
        );
        assert_eq!(
            key_from_character('f'),
            Some(Key::UnaryOperator(UnaryOperator::Factorial))
        );
        assert_eq!(
            key_from_character('^'),
            Some(Key::Operator(Operator::Power))
        );
        assert_eq!(
            key_from_character('p'),
            Some(Key::Constant(MathematicalConstant::Pi))
        );
        assert_eq!(
            key_from_character('k'),
            Some(Key::Constant(MathematicalConstant::Euler))
        );
        assert_eq!(key_from_character('('), Some(Key::OpenParenthesis));
        assert_eq!(key_from_character(')'), Some(Key::CloseParenthesis));
        assert_eq!(key_from_character('x'), None);
    }
}
