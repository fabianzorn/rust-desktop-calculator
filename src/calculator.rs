//! Arithmetic operations used by the calculator UI.

/// A binary arithmetic operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    /// Adds the second operand to the first.
    Add,
    /// Subtracts the second operand from the first.
    Subtract,
    /// Multiplies both operands.
    Multiply,
    /// Divides the first operand by the second.
    Divide,
}

/// An arithmetic operation that acts on a single value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Reverses the sign of the value.
    ToggleSign,
    /// Divides the value by one hundred.
    Percent,
    /// Calculates the square root of the value.
    SquareRoot,
    /// Multiplies the value by itself.
    Square,
    /// Calculates the sine of an angle in degrees.
    Sine,
    /// Calculates the cosine of an angle in degrees.
    Cosine,
    /// Calculates the tangent of an angle in degrees.
    Tangent,
}

impl Operator {
    /// Returns the symbol used to display this operator.
    pub fn symbol(self) -> char {
        match self {
            Self::Add => '+',
            Self::Subtract => '-',
            Self::Multiply => '*',
            Self::Divide => '/',
        }
    }
}

/// Applies a binary `operator` to `first` and `second`.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] when dividing by zero.
pub fn calculate(first: f64, operator: Operator, second: f64) -> Result<f64, CalculationError> {
    match operator {
        Operator::Add => Ok(first + second),
        Operator::Subtract => Ok(first - second),
        Operator::Multiply => Ok(first * second),
        Operator::Divide if second == 0.0 => Err(CalculationError::DivisionByZero),
        Operator::Divide => Ok(first / second),
    }
}

/// Applies a unary `operator` to `value`.
///
/// # Errors
///
/// Returns an error when calculating the square root of a negative value or
/// the tangent of an angle at which it is undefined.
pub fn calculate_unary(value: f64, operator: UnaryOperator) -> Result<f64, CalculationError> {
    match operator {
        UnaryOperator::ToggleSign => Ok(-value),
        UnaryOperator::Percent => Ok(value / 100.0),
        UnaryOperator::SquareRoot if value < 0.0 => Err(CalculationError::NegativeSquareRoot),
        UnaryOperator::SquareRoot => Ok(value.sqrt()),
        UnaryOperator::Square => Ok(value.powi(2)),
        UnaryOperator::Sine => Ok(round_trigonometric_result(value.to_radians().sin())),
        UnaryOperator::Cosine => Ok(round_trigonometric_result(value.to_radians().cos())),
        UnaryOperator::Tangent if tangent_is_undefined(value) => {
            Err(CalculationError::UndefinedTangent)
        }
        UnaryOperator::Tangent => Ok(round_trigonometric_result(value.to_radians().tan())),
    }
}

/// Reports whether the tangent is undefined for an angle in degrees.
fn tangent_is_undefined(degrees: f64) -> bool {
    degrees.to_radians().cos().abs() < f64::EPSILON.sqrt()
}

/// Rounds small floating-point artifacts produced by trigonometric functions.
fn round_trigonometric_result(value: f64) -> f64 {
    const PRECISION: f64 = 1_000_000_000_000.0;

    (value * PRECISION).round() / PRECISION
}

/// An error produced by an arithmetic operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalculationError {
    /// A division used zero as its second operand.
    DivisionByZero,
    /// A square root operation received a negative value.
    NegativeSquareRoot,
    /// A tangent operation received an angle whose cosine is zero.
    UndefinedTangent,
}

#[cfg(test)]
mod tests {
    use super::{CalculationError, Operator, UnaryOperator, calculate, calculate_unary};

    #[test]
    fn adds_numbers() {
        assert_eq!(calculate(8.0, Operator::Add, 4.0), Ok(12.0));
    }

    #[test]
    fn subtracts_numbers() {
        assert_eq!(calculate(8.0, Operator::Subtract, 4.0), Ok(4.0));
    }

    #[test]
    fn multiplies_numbers() {
        assert_eq!(calculate(8.0, Operator::Multiply, 4.0), Ok(32.0));
    }

    #[test]
    fn divides_numbers() {
        assert_eq!(calculate(8.0, Operator::Divide, 4.0), Ok(2.0));
    }

    #[test]
    fn rejects_division_by_zero() {
        assert_eq!(
            calculate(8.0, Operator::Divide, 0.0),
            Err(CalculationError::DivisionByZero)
        );
    }

    #[test]
    fn toggles_number_sign() {
        assert_eq!(calculate_unary(8.0, UnaryOperator::ToggleSign), Ok(-8.0));
        assert_eq!(calculate_unary(-8.0, UnaryOperator::ToggleSign), Ok(8.0));
    }

    #[test]
    fn calculates_percentage() {
        assert_eq!(calculate_unary(25.0, UnaryOperator::Percent), Ok(0.25));
    }

    #[test]
    fn calculates_square_root() {
        assert_eq!(calculate_unary(81.0, UnaryOperator::SquareRoot), Ok(9.0));
    }

    #[test]
    fn rejects_square_root_of_negative_number() {
        assert_eq!(
            calculate_unary(-4.0, UnaryOperator::SquareRoot),
            Err(CalculationError::NegativeSquareRoot)
        );
    }

    #[test]
    fn calculates_square() {
        assert_eq!(calculate_unary(12.0, UnaryOperator::Square), Ok(144.0));
    }

    #[test]
    fn calculates_sine_in_degrees() {
        assert_eq!(calculate_unary(30.0, UnaryOperator::Sine), Ok(0.5));
        assert_eq!(calculate_unary(90.0, UnaryOperator::Sine), Ok(1.0));
    }

    #[test]
    fn calculates_cosine_in_degrees() {
        assert_eq!(calculate_unary(60.0, UnaryOperator::Cosine), Ok(0.5));
        assert_eq!(calculate_unary(180.0, UnaryOperator::Cosine), Ok(-1.0));
    }

    #[test]
    fn calculates_tangent_in_degrees() {
        assert_eq!(calculate_unary(45.0, UnaryOperator::Tangent), Ok(1.0));
        assert_eq!(calculate_unary(180.0, UnaryOperator::Tangent), Ok(0.0));
    }

    #[test]
    fn rejects_tangent_at_undefined_angles() {
        assert_eq!(
            calculate_unary(90.0, UnaryOperator::Tangent),
            Err(CalculationError::UndefinedTangent)
        );
        assert_eq!(
            calculate_unary(270.0, UnaryOperator::Tangent),
            Err(CalculationError::UndefinedTangent)
        );
    }
}
