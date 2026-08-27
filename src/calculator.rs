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
    /// Raises the first operand to the power of the second.
    Power,
}

/// The unit used to interpret angles in trigonometric operations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AngleMode {
    /// Interprets angles as degrees.
    #[default]
    Degrees,
    /// Interprets angles as radians.
    Radians,
}

/// A mathematical constant that can be entered into the calculator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathematicalConstant {
    /// The ratio of a circle's circumference to its diameter.
    Pi,
    /// Euler's number, the base of the natural logarithm.
    Euler,
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
    /// Calculates the sine of an angle in the selected unit.
    Sine,
    /// Calculates the cosine of an angle in the selected unit.
    Cosine,
    /// Calculates the tangent of an angle in the selected unit.
    Tangent,
    /// Calculates the base-10 logarithm of the value.
    LogarithmBase10,
    /// Calculates the natural logarithm of the value.
    NaturalLogarithm,
    /// Raises Euler's number to the value.
    Exponential,
    /// Calculates one divided by the value.
    Reciprocal,
    /// Calculates the factorial of a non-negative integer.
    Factorial,
}

impl Operator {
    /// Returns the symbol used to display this operator.
    pub fn symbol(self) -> char {
        match self {
            Self::Add => '+',
            Self::Subtract => '-',
            Self::Multiply => '*',
            Self::Divide => '/',
            Self::Power => '^',
        }
    }
}

impl AngleMode {
    /// Returns the short label displayed by the angle-mode button.
    pub fn label(self) -> &'static str {
        match self {
            Self::Degrees => "DEG",
            Self::Radians => "RAD",
        }
    }

    /// Returns the other available angle mode.
    pub fn toggled(self) -> Self {
        match self {
            Self::Degrees => Self::Radians,
            Self::Radians => Self::Degrees,
        }
    }

    /// Converts `value` from this angle unit to radians.
    fn radians(self, value: f64) -> f64 {
        match self {
            Self::Degrees => value.to_radians(),
            Self::Radians => value,
        }
    }
}

impl MathematicalConstant {
    /// Returns the constant's full-precision [`f64`] value.
    pub fn value(self) -> f64 {
        match self {
            Self::Pi => std::f64::consts::PI,
            Self::Euler => std::f64::consts::E,
        }
    }

    /// Returns the symbol displayed by the calculator.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Pi => "π",
            Self::Euler => "e",
        }
    }
}

/// Applies a binary `operator` to `first` and `second`.
///
/// # Errors
///
/// Returns an error when dividing by zero or when a power has no finite real
/// result.
pub fn calculate(first: f64, operator: Operator, second: f64) -> Result<f64, CalculationError> {
    match operator {
        Operator::Add => Ok(first + second),
        Operator::Subtract => Ok(first - second),
        Operator::Multiply => Ok(first * second),
        Operator::Divide if second == 0.0 => Err(CalculationError::DivisionByZero),
        Operator::Divide => Ok(first / second),
        Operator::Power if first == 0.0 && second < 0.0 => Err(CalculationError::DivisionByZero),
        Operator::Power => finite_result(first.powf(second), CalculationError::InvalidPower),
    }
}

/// Applies a unary `operator` to `value`, using `angle_mode` for trigonometry.
///
/// # Errors
///
/// Returns an error when the operation is undefined for `value` or its result
/// is outside the finite range represented by [`f64`].
pub fn calculate_unary(
    value: f64,
    operator: UnaryOperator,
    angle_mode: AngleMode,
) -> Result<f64, CalculationError> {
    match operator {
        UnaryOperator::ToggleSign => Ok(-value),
        UnaryOperator::Percent => Ok(value / 100.0),
        UnaryOperator::SquareRoot if value < 0.0 => Err(CalculationError::NegativeSquareRoot),
        UnaryOperator::SquareRoot => Ok(value.sqrt()),
        UnaryOperator::Square => Ok(value.powi(2)),
        UnaryOperator::Sine => Ok(round_trigonometric_result(angle_mode.radians(value).sin())),
        UnaryOperator::Cosine => Ok(round_trigonometric_result(angle_mode.radians(value).cos())),
        UnaryOperator::Tangent if tangent_is_undefined(value, angle_mode) => {
            Err(CalculationError::UndefinedTangent)
        }
        UnaryOperator::Tangent => Ok(round_trigonometric_result(angle_mode.radians(value).tan())),
        UnaryOperator::LogarithmBase10 if value <= 0.0 => Err(CalculationError::InvalidLogarithm),
        UnaryOperator::LogarithmBase10 => Ok(value.log10()),
        UnaryOperator::NaturalLogarithm if value <= 0.0 => Err(CalculationError::InvalidLogarithm),
        UnaryOperator::NaturalLogarithm => Ok(value.ln()),
        UnaryOperator::Exponential => {
            finite_result(value.exp(), CalculationError::ResultOutOfRange)
        }
        UnaryOperator::Reciprocal if value == 0.0 => Err(CalculationError::DivisionByZero),
        UnaryOperator::Reciprocal => Ok(value.recip()),
        UnaryOperator::Factorial if value < 0.0 || value.fract() != 0.0 => {
            Err(CalculationError::InvalidFactorial)
        }
        UnaryOperator::Factorial if value > 170.0 => Err(CalculationError::ResultOutOfRange),
        UnaryOperator::Factorial => Ok(factorial(value as u32)),
    }
}

/// Returns a finite calculation result or the supplied domain/range error.
fn finite_result(result: f64, error: CalculationError) -> Result<f64, CalculationError> {
    if result.is_finite() {
        Ok(result)
    } else {
        Err(error)
    }
}

/// Calculates a factorial that is known to fit into an [`f64`].
fn factorial(value: u32) -> f64 {
    (1..=value).fold(1.0, |result, factor| result * f64::from(factor))
}

/// Reports whether the tangent is undefined for an angle in the selected unit.
fn tangent_is_undefined(value: f64, angle_mode: AngleMode) -> bool {
    angle_mode.radians(value).cos().abs() < f64::EPSILON.sqrt()
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
    /// A logarithm received zero or a negative value.
    InvalidLogarithm,
    /// A power has no finite result in the real number domain.
    InvalidPower,
    /// A factorial received a negative or non-integer value.
    InvalidFactorial,
    /// A result exceeds the finite range represented by [`f64`].
    ResultOutOfRange,
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{E, FRAC_PI_2, FRAC_PI_4, PI};

    use super::{
        AngleMode, CalculationError, MathematicalConstant, Operator, UnaryOperator, calculate,
        calculate_unary,
    };

    #[test]
    fn provides_mathematical_constants() {
        assert_eq!(MathematicalConstant::Pi.value(), PI);
        assert_eq!(MathematicalConstant::Pi.symbol(), "π");
        assert_eq!(MathematicalConstant::Euler.value(), E);
        assert_eq!(MathematicalConstant::Euler.symbol(), "e");
    }

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
    fn calculates_arbitrary_power() {
        assert_eq!(calculate(2.0, Operator::Power, 10.0), Ok(1024.0));
        assert_eq!(calculate(9.0, Operator::Power, 0.5), Ok(3.0));
    }

    #[test]
    fn rejects_invalid_power() {
        assert_eq!(
            calculate(-2.0, Operator::Power, 0.5),
            Err(CalculationError::InvalidPower)
        );
        assert_eq!(
            calculate(0.0, Operator::Power, -1.0),
            Err(CalculationError::DivisionByZero)
        );
    }

    #[test]
    fn toggles_number_sign() {
        assert_eq!(
            calculate_unary(8.0, UnaryOperator::ToggleSign, AngleMode::Degrees),
            Ok(-8.0)
        );
        assert_eq!(
            calculate_unary(-8.0, UnaryOperator::ToggleSign, AngleMode::Degrees),
            Ok(8.0)
        );
    }

    #[test]
    fn calculates_percentage() {
        assert_eq!(
            calculate_unary(25.0, UnaryOperator::Percent, AngleMode::Degrees),
            Ok(0.25)
        );
    }

    #[test]
    fn calculates_square_root() {
        assert_eq!(
            calculate_unary(81.0, UnaryOperator::SquareRoot, AngleMode::Degrees),
            Ok(9.0)
        );
    }

    #[test]
    fn rejects_square_root_of_negative_number() {
        assert_eq!(
            calculate_unary(-4.0, UnaryOperator::SquareRoot, AngleMode::Degrees),
            Err(CalculationError::NegativeSquareRoot)
        );
    }

    #[test]
    fn calculates_square() {
        assert_eq!(
            calculate_unary(12.0, UnaryOperator::Square, AngleMode::Degrees),
            Ok(144.0)
        );
    }

    #[test]
    fn calculates_logarithms() {
        assert_eq!(
            calculate_unary(1000.0, UnaryOperator::LogarithmBase10, AngleMode::Degrees),
            Ok(3.0)
        );
        assert_eq!(
            calculate_unary(
                std::f64::consts::E,
                UnaryOperator::NaturalLogarithm,
                AngleMode::Degrees
            ),
            Ok(1.0)
        );
    }

    #[test]
    fn rejects_non_positive_logarithm_arguments() {
        for operator in [
            UnaryOperator::LogarithmBase10,
            UnaryOperator::NaturalLogarithm,
        ] {
            assert_eq!(
                calculate_unary(0.0, operator, AngleMode::Degrees),
                Err(CalculationError::InvalidLogarithm)
            );
            assert_eq!(
                calculate_unary(-1.0, operator, AngleMode::Degrees),
                Err(CalculationError::InvalidLogarithm)
            );
        }
    }

    #[test]
    fn calculates_exponential_and_reciprocal() {
        assert_eq!(
            calculate_unary(0.0, UnaryOperator::Exponential, AngleMode::Degrees),
            Ok(1.0)
        );
        assert_eq!(
            calculate_unary(4.0, UnaryOperator::Reciprocal, AngleMode::Degrees),
            Ok(0.25)
        );
        assert_eq!(
            calculate_unary(0.0, UnaryOperator::Reciprocal, AngleMode::Degrees),
            Err(CalculationError::DivisionByZero)
        );
    }

    #[test]
    fn calculates_factorials() {
        assert_eq!(
            calculate_unary(0.0, UnaryOperator::Factorial, AngleMode::Degrees),
            Ok(1.0)
        );
        assert_eq!(
            calculate_unary(5.0, UnaryOperator::Factorial, AngleMode::Degrees),
            Ok(120.0)
        );
    }

    #[test]
    fn rejects_invalid_or_out_of_range_factorials() {
        assert_eq!(
            calculate_unary(-1.0, UnaryOperator::Factorial, AngleMode::Degrees),
            Err(CalculationError::InvalidFactorial)
        );
        assert_eq!(
            calculate_unary(2.5, UnaryOperator::Factorial, AngleMode::Degrees),
            Err(CalculationError::InvalidFactorial)
        );
        assert_eq!(
            calculate_unary(171.0, UnaryOperator::Factorial, AngleMode::Degrees),
            Err(CalculationError::ResultOutOfRange)
        );
    }

    #[test]
    fn calculates_sine_in_degrees() {
        assert_eq!(
            calculate_unary(30.0, UnaryOperator::Sine, AngleMode::Degrees),
            Ok(0.5)
        );
        assert_eq!(
            calculate_unary(90.0, UnaryOperator::Sine, AngleMode::Degrees),
            Ok(1.0)
        );
    }

    #[test]
    fn calculates_cosine_in_degrees() {
        assert_eq!(
            calculate_unary(60.0, UnaryOperator::Cosine, AngleMode::Degrees),
            Ok(0.5)
        );
        assert_eq!(
            calculate_unary(180.0, UnaryOperator::Cosine, AngleMode::Degrees),
            Ok(-1.0)
        );
    }

    #[test]
    fn calculates_tangent_in_degrees() {
        assert_eq!(
            calculate_unary(45.0, UnaryOperator::Tangent, AngleMode::Degrees),
            Ok(1.0)
        );
        assert_eq!(
            calculate_unary(180.0, UnaryOperator::Tangent, AngleMode::Degrees),
            Ok(0.0)
        );
    }

    #[test]
    fn rejects_tangent_at_undefined_angles() {
        assert_eq!(
            calculate_unary(90.0, UnaryOperator::Tangent, AngleMode::Degrees),
            Err(CalculationError::UndefinedTangent)
        );
        assert_eq!(
            calculate_unary(270.0, UnaryOperator::Tangent, AngleMode::Degrees),
            Err(CalculationError::UndefinedTangent)
        );
    }

    #[test]
    fn calculates_trigonometry_in_radians() {
        assert_eq!(
            calculate_unary(FRAC_PI_2, UnaryOperator::Sine, AngleMode::Radians),
            Ok(1.0)
        );
        assert_eq!(
            calculate_unary(PI, UnaryOperator::Cosine, AngleMode::Radians),
            Ok(-1.0)
        );
        assert_eq!(
            calculate_unary(FRAC_PI_4, UnaryOperator::Tangent, AngleMode::Radians),
            Ok(1.0)
        );
    }

    #[test]
    fn rejects_undefined_tangent_in_radians() {
        assert_eq!(
            calculate_unary(FRAC_PI_2, UnaryOperator::Tangent, AngleMode::Radians),
            Err(CalculationError::UndefinedTangent)
        );
    }

    #[test]
    fn toggles_angle_mode() {
        assert_eq!(AngleMode::Degrees.toggled(), AngleMode::Radians);
        assert_eq!(AngleMode::Radians.toggled(), AngleMode::Degrees);
        assert_eq!(AngleMode::Degrees.label(), "DEG");
        assert_eq!(AngleMode::Radians.label(), "RAD");
    }
}
