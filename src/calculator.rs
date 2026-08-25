#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operator {
    pub fn symbol(self) -> char {
        match self {
            Self::Add => '+',
            Self::Subtract => '-',
            Self::Multiply => '*',
            Self::Divide => '/',
        }
    }
}

pub fn calculate(first: f64, operator: Operator, second: f64) -> Result<f64, CalculationError> {
    match operator {
        Operator::Add => Ok(first + second),
        Operator::Subtract => Ok(first - second),
        Operator::Multiply => Ok(first * second),
        Operator::Divide if second == 0.0 => Err(CalculationError::DivisionByZero),
        Operator::Divide => Ok(first / second),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalculationError {
    DivisionByZero,
}

#[cfg(test)]
mod tests {
    use super::{CalculationError, Operator, calculate};

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
}
