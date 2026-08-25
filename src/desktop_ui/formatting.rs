//! Formatting helpers for values and expressions shown by the UI.

use crate::calculator::UnaryOperator;

/// Formats a numeric result for the main calculator display.
pub(super) fn format_number(number: f64) -> String {
    if number == 0.0 {
        return "0".to_owned();
    }

    let formatted = number.to_string();

    if formatted.ends_with(".0") {
        formatted.trim_end_matches(".0").to_owned()
    } else {
        formatted
    }
}

/// Formats a unary operation for the expression display.
pub(super) fn format_unary_expression(value: f64, operator: UnaryOperator) -> String {
    let value = format_number(value);

    match operator {
        UnaryOperator::ToggleSign => format!("-({value})"),
        UnaryOperator::Percent => format!("{value}%"),
        UnaryOperator::SquareRoot => format!("√({value})"),
        UnaryOperator::Square => format!("({value})²"),
    }
}

/// Returns a non-empty placeholder for an empty expression.
///
/// Keeping one blank character prevents the display layout from changing
/// height while no expression is active.
pub(super) fn display_expression(expression: &str) -> &str {
    if expression.is_empty() {
        " "
    } else {
        expression
    }
}

/// Chooses a font size that keeps long values inside the display.
pub(super) fn display_size(display: &str) -> f32 {
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

    #[test]
    fn formats_integer_results_without_decimal_suffix() {
        assert_eq!(format_number(12.0), "12");
    }

    #[test]
    fn keeps_fractional_results() {
        assert_eq!(format_number(2.5), "2.5");
    }

    #[test]
    fn normalizes_negative_zero() {
        assert_eq!(format_number(-0.0), "0");
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
