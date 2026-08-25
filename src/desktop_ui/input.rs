use eframe::egui;

use crate::calculator::{Operator, UnaryOperator};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Key {
    Number(char),
    Decimal,
    Operator(Operator),
    UnaryOperator(UnaryOperator),
    Equals,
    Backspace,
    Clear,
}

pub(super) fn keyboard_keys(context: &egui::Context) -> Vec<Key> {
    context.input(|input| {
        let mut keys = input
            .events
            .iter()
            .filter_map(|event| match event {
                egui::Event::Text(text) => text.chars().find_map(key_from_character),
                _ => None,
            })
            .collect::<Vec<_>>();

        if input.key_pressed(egui::Key::Enter) {
            keys.push(Key::Equals);
        }
        if input.key_pressed(egui::Key::Backspace) {
            keys.push(Key::Backspace);
        }
        if input.key_pressed(egui::Key::Escape) || input.key_pressed(egui::Key::Delete) {
            keys.push(Key::Clear);
        }

        keys
    })
}

fn key_from_character(character: char) -> Option<Key> {
    match character {
        '0'..='9' => Some(Key::Number(character)),
        '.' | ',' => Some(Key::Decimal),
        '+' => Some(Key::Operator(Operator::Add)),
        '-' => Some(Key::Operator(Operator::Subtract)),
        '*' | '×' => Some(Key::Operator(Operator::Multiply)),
        '/' | '÷' => Some(Key::Operator(Operator::Divide)),
        '=' => Some(Key::Equals),
        '%' => Some(Key::UnaryOperator(UnaryOperator::Percent)),
        'r' | 'R' => Some(Key::UnaryOperator(UnaryOperator::SquareRoot)),
        's' | 'S' => Some(Key::UnaryOperator(UnaryOperator::Square)),
        'n' | 'N' => Some(Key::UnaryOperator(UnaryOperator::ToggleSign)),
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
        assert_eq!(key_from_character('x'), None);
    }
}
