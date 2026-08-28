//! Translation of keyboard events into calculator actions.

use eframe::egui;

use crate::calculator::{MathematicalConstant, Operator, UnaryOperator};

use super::programmer::{NumberBase, ProgrammerKey, ProgrammerOperator};

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
        let mut keys = collect_text_and_edit_keys(
            input,
            input.modifiers.command,
            key_from_character,
            Key::Equals,
            Key::Backspace,
            Key::Clear,
        );
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

/// Collects programmer-mode actions triggered during the current input frame.
pub(super) fn programmer_keyboard_keys(context: &egui::Context) -> Vec<ProgrammerKey> {
    context.input(|input| {
        let mut keys = collect_text_and_edit_keys(
            input,
            input.modifiers.command || input.modifiers.ctrl,
            programmer_key_from_character,
            ProgrammerKey::Equals,
            ProgrammerKey::Backspace,
            ProgrammerKey::Clear,
        );
        if input.modifiers.ctrl && input.key_pressed(egui::Key::B) {
            keys.push(ProgrammerKey::SetBase(NumberBase::Binary));
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::O) {
            keys.push(ProgrammerKey::SetBase(NumberBase::Octal));
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::D) {
            keys.push(ProgrammerKey::SetBase(NumberBase::Decimal));
        }
        if input.modifiers.ctrl && input.key_pressed(egui::Key::H) {
            keys.push(ProgrammerKey::SetBase(NumberBase::Hexadecimal));
        }

        keys
    })
}

/// Collects text input and the editing keys shared by both calculator modes.
fn collect_text_and_edit_keys<K: Copy>(
    input: &egui::InputState,
    suppress_text_input: bool,
    map_character: fn(char) -> Option<K>,
    equals: K,
    backspace: K,
    clear: K,
) -> Vec<K> {
    let mut keys = if suppress_text_input {
        Vec::new()
    } else {
        input
            .events
            .iter()
            .filter_map(|event| match event {
                egui::Event::Text(text) => text.chars().find_map(map_character),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    if input.key_pressed(egui::Key::Enter) {
        keys.push(equals);
    }
    if input.key_pressed(egui::Key::Backspace) {
        keys.push(backspace);
    }
    if input.key_pressed(egui::Key::Escape) || input.key_pressed(egui::Key::Delete) {
        keys.push(clear);
    }

    keys
}

/// Maps typed characters to programmer-mode actions.
fn programmer_key_from_character(character: char) -> Option<ProgrammerKey> {
    match character {
        '0'..='9' => Some(ProgrammerKey::Digit(character as u8 - b'0')),
        'a'..='f' => Some(ProgrammerKey::Digit(character as u8 - b'a' + 10)),
        'A'..='F' => Some(ProgrammerKey::Digit(character as u8 - b'A' + 10)),
        '+' => Some(ProgrammerKey::Operator(ProgrammerOperator::Add)),
        '-' => Some(ProgrammerKey::Operator(ProgrammerOperator::Subtract)),
        '*' | '×' => Some(ProgrammerKey::Operator(ProgrammerOperator::Multiply)),
        '/' | '÷' => Some(ProgrammerKey::Operator(ProgrammerOperator::Divide)),
        '%' => Some(ProgrammerKey::Operator(ProgrammerOperator::Modulo)),
        '&' => Some(ProgrammerKey::Operator(ProgrammerOperator::And)),
        '|' => Some(ProgrammerKey::Operator(ProgrammerOperator::Or)),
        '^' => Some(ProgrammerKey::Operator(ProgrammerOperator::Xor)),
        '<' => Some(ProgrammerKey::Operator(ProgrammerOperator::ShiftLeft)),
        '>' => Some(ProgrammerKey::Operator(ProgrammerOperator::ShiftRight)),
        '~' => Some(ProgrammerKey::Not),
        '=' => Some(ProgrammerKey::Equals),
        _ => None,
    }
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
        'h' | 'H' => Some(Key::UnaryOperator(UnaryOperator::HyperbolicSine)),
        'u' | 'U' => Some(Key::UnaryOperator(UnaryOperator::HyperbolicCosine)),
        'y' | 'Y' => Some(Key::UnaryOperator(UnaryOperator::HyperbolicTangent)),
        'a' | 'A' => Some(Key::UnaryOperator(UnaryOperator::AbsoluteValue)),
        'g' | 'G' => Some(Key::UnaryOperator(UnaryOperator::Floor)),
        'b' | 'B' => Some(Key::UnaryOperator(UnaryOperator::Ceiling)),
        'd' | 'D' => Some(Key::Operator(Operator::Modulo)),
        'j' | 'J' => Some(Key::Operator(Operator::ScientificNotation)),
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
        assert_eq!(
            key_from_character('h'),
            Some(Key::UnaryOperator(UnaryOperator::HyperbolicSine))
        );
        assert_eq!(
            key_from_character('u'),
            Some(Key::UnaryOperator(UnaryOperator::HyperbolicCosine))
        );
        assert_eq!(
            key_from_character('y'),
            Some(Key::UnaryOperator(UnaryOperator::HyperbolicTangent))
        );
        assert_eq!(
            key_from_character('a'),
            Some(Key::UnaryOperator(UnaryOperator::AbsoluteValue))
        );
        assert_eq!(
            key_from_character('g'),
            Some(Key::UnaryOperator(UnaryOperator::Floor))
        );
        assert_eq!(
            key_from_character('b'),
            Some(Key::UnaryOperator(UnaryOperator::Ceiling))
        );
        assert_eq!(
            key_from_character('d'),
            Some(Key::Operator(Operator::Modulo))
        );
        assert_eq!(
            key_from_character('j'),
            Some(Key::Operator(Operator::ScientificNotation))
        );
        assert_eq!(key_from_character('x'), None);
    }

    #[test]
    fn maps_programmer_keyboard_characters() {
        assert_eq!(
            programmer_key_from_character('F'),
            Some(ProgrammerKey::Digit(15))
        );
        assert_eq!(
            programmer_key_from_character('&'),
            Some(ProgrammerKey::Operator(ProgrammerOperator::And))
        );
        assert_eq!(programmer_key_from_character('~'), Some(ProgrammerKey::Not));
        assert_eq!(programmer_key_from_character('G'), None);
    }
}
