//! Integer, number-base, and bitwise logic for programmer mode.

/// A number base supported by programmer mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum NumberBase {
    /// Base two.
    Binary,
    /// Base eight.
    Octal,
    /// Base ten.
    #[default]
    Decimal,
    /// Base sixteen.
    Hexadecimal,
}

impl NumberBase {
    /// Returns the short label shown by the base selector.
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Binary => "BIN",
            Self::Octal => "OCT",
            Self::Decimal => "DEC",
            Self::Hexadecimal => "HEX",
        }
    }

    /// Returns the radix represented by this base.
    fn radix(self) -> u32 {
        match self {
            Self::Binary => 2,
            Self::Octal => 8,
            Self::Decimal => 10,
            Self::Hexadecimal => 16,
        }
    }

    /// Reports whether a numeric digit can be entered in this base.
    pub(super) fn accepts(self, digit: u8) -> bool {
        u32::from(digit) < self.radix()
    }
}

/// Integer width used to mask programmer-mode calculations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum WordSize {
    /// Eight-bit integers.
    Bits8,
    /// Sixteen-bit integers.
    Bits16,
    /// Thirty-two-bit integers.
    Bits32,
    /// Sixty-four-bit integers.
    #[default]
    Bits64,
}

impl WordSize {
    /// Returns the number of bits in this word size.
    pub(super) fn bits(self) -> u32 {
        match self {
            Self::Bits8 => 8,
            Self::Bits16 => 16,
            Self::Bits32 => 32,
            Self::Bits64 => 64,
        }
    }

    /// Returns the word-size label shown by the selector.
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Bits8 => "8-bit",
            Self::Bits16 => "16-bit",
            Self::Bits32 => "32-bit",
            Self::Bits64 => "64-bit",
        }
    }

    /// Returns a mask containing one bit for every bit in this word size.
    fn mask(self) -> u64 {
        match self {
            Self::Bits64 => u64::MAX,
            _ => (1_u64 << self.bits()) - 1,
        }
    }
}

/// A binary operation supported by programmer mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProgrammerOperator {
    /// Wrapping addition.
    Add,
    /// Wrapping subtraction.
    Subtract,
    /// Wrapping multiplication.
    Multiply,
    /// Integer division.
    Divide,
    /// Integer remainder.
    Modulo,
    /// Bitwise AND.
    And,
    /// Bitwise OR.
    Or,
    /// Bitwise exclusive OR.
    Xor,
    /// Logical left shift.
    ShiftLeft,
    /// Logical right shift.
    ShiftRight,
}

impl ProgrammerOperator {
    /// Returns the operator label shown in expressions and on buttons.
    pub(super) fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "−",
            Self::Multiply => "×",
            Self::Divide => "÷",
            Self::Modulo => "mod",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Xor => "XOR",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
        }
    }

    /// Applies this operation using the wrapping semantics of `word_size`.
    fn calculate(
        self,
        first: u64,
        second: u64,
        word_size: WordSize,
    ) -> Result<u64, ProgrammerCalculationError> {
        let mask = word_size.mask();
        let result = match self {
            Self::Add => first.wrapping_add(second),
            Self::Subtract => first.wrapping_sub(second),
            Self::Multiply => first.wrapping_mul(second),
            Self::Divide | Self::Modulo if second == 0 => {
                return Err(ProgrammerCalculationError::DivisionByZero);
            }
            Self::Divide => first / second,
            Self::Modulo => first % second,
            Self::And => first & second,
            Self::Or => first | second,
            Self::Xor => first ^ second,
            Self::ShiftLeft if second >= u64::from(word_size.bits()) => 0,
            Self::ShiftLeft => first.wrapping_shl(second as u32),
            Self::ShiftRight if second >= u64::from(word_size.bits()) => 0,
            Self::ShiftRight => first.wrapping_shr(second as u32),
        };

        Ok(result & mask)
    }
}

/// An error produced by a programmer-mode integer operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProgrammerCalculationError {
    DivisionByZero,
}

impl ProgrammerCalculationError {
    /// Returns the message shown in the programmer display.
    fn message(self) -> &'static str {
        match self {
            Self::DivisionByZero => "Cannot divide by zero",
        }
    }
}

/// An input action understood by programmer mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProgrammerKey {
    /// Enters one hexadecimal digit from zero through fifteen.
    Digit(u8),
    /// Selects a binary integer operation.
    Operator(ProgrammerOperator),
    /// Evaluates the pending operation.
    Equals,
    /// Clears programmer-mode calculation state.
    Clear,
    /// Removes the least-significant entered digit.
    Backspace,
    /// Calculates the bitwise complement.
    Not,
    /// Calculates the one's complement.
    OnesComplement,
    /// Calculates the two's complement.
    TwosComplement,
    /// Selects an input and display base.
    SetBase(NumberBase),
}

/// Owns the integer calculation state used by programmer mode.
#[derive(Default)]
pub(super) struct ProgrammerState {
    value: u64,
    first_value: Option<u64>,
    operator: Option<ProgrammerOperator>,
    waiting_for_second_value: bool,
    base: NumberBase,
    word_size: WordSize,
    expression: String,
    error: Option<&'static str>,
}

impl ProgrammerState {
    /// Applies one button or keyboard action.
    pub(super) fn handle_key(&mut self, key: ProgrammerKey) {
        match key {
            ProgrammerKey::Digit(digit) => self.append_digit(digit),
            ProgrammerKey::Operator(operator) => self.choose_operator(operator),
            ProgrammerKey::Equals => self.calculate_result(),
            ProgrammerKey::Clear => self.clear_calculation(),
            ProgrammerKey::Backspace => self.backspace(),
            ProgrammerKey::Not => self.complement("NOT"),
            ProgrammerKey::OnesComplement => self.complement("1's complement"),
            ProgrammerKey::TwosComplement => self.twos_complement(),
            ProgrammerKey::SetBase(base) => self.set_base(base),
        }
    }

    /// Returns the currently selected number base.
    pub(super) fn base(&self) -> NumberBase {
        self.base
    }

    /// Returns the selected integer word size.
    pub(super) fn word_size(&self) -> WordSize {
        self.word_size
    }

    /// Changes the integer word size and masks existing operands accordingly.
    pub(super) fn set_word_size(&mut self, word_size: WordSize) {
        self.word_size = word_size;
        self.value &= word_size.mask();
        self.first_value = self.first_value.map(|value| value & word_size.mask());
        self.update_expression();
    }

    /// Returns the main display text in the active number base.
    pub(super) fn display(&self) -> String {
        self.error
            .map_or_else(|| format_value(self.value, self.base), ToOwned::to_owned)
    }

    /// Returns the pending integer expression.
    pub(super) fn expression(&self) -> &str {
        &self.expression
    }

    /// Reports whether the state currently contains an error.
    pub(super) fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Formats the current value in any supported number base.
    pub(super) fn conversion(&self, base: NumberBase) -> String {
        format_value(self.value, base)
    }

    /// Formats the current bit pattern as a signed two's-complement decimal value.
    pub(super) fn signed_decimal_conversion(&self) -> String {
        let bits = self.word_size.bits();
        if bits == 64 {
            (self.value as i64).to_string()
        } else {
            let sign_bit = 1_u64 << (bits - 1);
            if self.value & sign_bit == 0 {
                self.value.to_string()
            } else {
                (i128::from(self.value) - (1_i128 << bits)).to_string()
            }
        }
    }

    /// Returns the current unsigned integer value.
    #[cfg(test)]
    pub(super) fn value(&self) -> u64 {
        self.value
    }

    /// Reports whether a bit is available at the selected word size.
    pub(super) fn bit_is_available(&self, bit: u32) -> bool {
        bit < self.word_size.bits()
    }

    /// Reports whether a bit is currently set.
    pub(super) fn bit_is_set(&self, bit: u32) -> bool {
        self.bit_is_available(bit) && self.value & (1_u64 << bit) != 0
    }

    /// Toggles one available bit in the current value.
    pub(super) fn toggle_bit(&mut self, bit: u32) {
        if self.error.is_none() && self.bit_is_available(bit) {
            if self.waiting_for_second_value {
                self.value = 0;
                self.waiting_for_second_value = false;
            }
            self.value ^= 1_u64 << bit;
            self.update_expression();
        }
    }

    /// Reports whether an operator is selected and waiting for its second operand.
    pub(super) fn is_active_operator(&self, operator: ProgrammerOperator) -> bool {
        self.operator == Some(operator) && self.waiting_for_second_value
    }

    fn append_digit(&mut self, digit: u8) {
        self.clear_error();
        if !self.base.accepts(digit) {
            return;
        }
        if self.waiting_for_second_value {
            self.value = 0;
            self.waiting_for_second_value = false;
        }

        let radix = u64::from(self.base.radix());
        let Some(value) = self
            .value
            .checked_mul(radix)
            .and_then(|value| value.checked_add(u64::from(digit)))
        else {
            self.show_error("Overflow");
            return;
        };
        if value > self.word_size.mask() {
            self.show_error("Overflow");
            return;
        }
        self.value = value;
        self.update_expression();
    }

    fn choose_operator(&mut self, operator: ProgrammerOperator) {
        self.clear_error();
        if self.operator.is_some() && !self.waiting_for_second_value {
            self.calculate_result();
        }
        if self.error.is_some() {
            return;
        }

        self.first_value = Some(self.value);
        self.operator = Some(operator);
        self.waiting_for_second_value = true;
        self.update_expression();
    }

    fn calculate_result(&mut self) {
        let (Some(first), Some(operator)) = (self.first_value, self.operator) else {
            return;
        };
        if self.waiting_for_second_value {
            return;
        }
        let second = self.value;
        self.expression = format!(
            "{} {} {}",
            format_value(first, self.base),
            operator.symbol(),
            format_value(second, self.base)
        );

        let result = match operator.calculate(first, second, self.word_size) {
            Ok(result) => result,
            Err(error) => {
                self.show_error(error.message());
                return;
            }
        };

        self.value = result;
        self.first_value = None;
        self.operator = None;
        self.waiting_for_second_value = false;
    }

    fn complement(&mut self, expression_label: &str) {
        if self.error.is_none() && !self.waiting_for_second_value {
            let original = self.value;
            self.value = !self.value & self.word_size.mask();
            self.expression = format!("{expression_label} {}", format_value(original, self.base));
        }
    }

    fn twos_complement(&mut self) {
        if self.error.is_none() && !self.waiting_for_second_value {
            let original = self.value;
            self.value = (!self.value).wrapping_add(1) & self.word_size.mask();
            self.expression = format!("2's complement {}", format_value(original, self.base));
        }
    }

    fn backspace(&mut self) {
        self.clear_error();
        if !self.waiting_for_second_value {
            self.value /= u64::from(self.base.radix());
            self.update_expression();
        }
    }

    fn set_base(&mut self, base: NumberBase) {
        self.base = base;
        self.update_expression();
    }

    fn clear_error(&mut self) {
        if self.error.is_some() {
            self.clear_calculation();
        }
    }

    fn clear_calculation(&mut self) {
        let base = self.base;
        let word_size = self.word_size;
        *self = Self {
            base,
            word_size,
            ..Self::default()
        };
    }

    fn show_error(&mut self, message: &'static str) {
        self.error = Some(message);
        self.first_value = None;
        self.operator = None;
        self.waiting_for_second_value = false;
    }

    fn update_expression(&mut self) {
        match (self.first_value, self.operator) {
            (Some(first), Some(operator)) if self.waiting_for_second_value => {
                self.expression =
                    format!("{} {}", format_value(first, self.base), operator.symbol());
            }
            (Some(first), Some(operator)) => {
                self.expression = format!(
                    "{} {} {}",
                    format_value(first, self.base),
                    operator.symbol(),
                    format_value(self.value, self.base)
                );
            }
            _ => self.expression.clear(),
        }
    }
}

/// Formats an unsigned integer using uppercase hexadecimal and grouped binary digits.
fn format_value(value: u64, base: NumberBase) -> String {
    match base {
        NumberBase::Binary => group_binary_digits(&format!("{value:b}")),
        NumberBase::Octal => format!("{value:o}"),
        NumberBase::Decimal => value.to_string(),
        NumberBase::Hexadecimal => format!("{value:X}"),
    }
}

/// Groups binary digits in nibbles for easier scanning.
fn group_binary_digits(digits: &str) -> String {
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 4);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(4) {
            grouped.push(' ');
        }
        grouped.push(digit);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enter(state: &mut ProgrammerState, digits: &[u8]) {
        for digit in digits {
            state.handle_key(ProgrammerKey::Digit(*digit));
        }
    }

    #[test]
    fn converts_octal_to_all_supported_bases() {
        let mut state = ProgrammerState::default();
        state.handle_key(ProgrammerKey::SetBase(NumberBase::Octal));
        enter(&mut state, &[1, 7]);

        assert_eq!(state.value(), 15);
        assert_eq!(state.conversion(NumberBase::Binary), "1111");
        assert_eq!(state.conversion(NumberBase::Octal), "17");
        assert_eq!(state.conversion(NumberBase::Decimal), "15");
        assert_eq!(state.conversion(NumberBase::Hexadecimal), "F");
    }

    #[test]
    fn accepts_hexadecimal_digits_only_in_compatible_bases() {
        let mut state = ProgrammerState::default();
        state.handle_key(ProgrammerKey::Digit(10));
        assert_eq!(state.value(), 0);

        state.handle_key(ProgrammerKey::SetBase(NumberBase::Hexadecimal));
        enter(&mut state, &[10, 15]);
        assert_eq!(state.value(), 0xAF);
        assert_eq!(state.display(), "AF");
    }

    #[test]
    fn calculates_integer_arithmetic_and_modulo() {
        let cases = [
            (ProgrammerOperator::Add, 13),
            (ProgrammerOperator::Subtract, 7),
            (ProgrammerOperator::Multiply, 30),
            (ProgrammerOperator::Divide, 3),
            (ProgrammerOperator::Modulo, 1),
        ];
        for (operator, expected) in cases {
            let mut state = ProgrammerState::default();
            enter(&mut state, &[1, 0]);
            state.handle_key(ProgrammerKey::Operator(operator));
            state.handle_key(ProgrammerKey::Digit(3));
            state.handle_key(ProgrammerKey::Equals);
            assert_eq!(state.value(), expected);
        }
    }

    #[test]
    fn calculates_bitwise_operations() {
        let cases = [
            (ProgrammerOperator::And, 0b1000),
            (ProgrammerOperator::Or, 0b1110),
            (ProgrammerOperator::Xor, 0b0110),
        ];
        for (operator, expected) in cases {
            let mut state = ProgrammerState::default();
            state.handle_key(ProgrammerKey::SetBase(NumberBase::Binary));
            enter(&mut state, &[1, 0, 1, 0]);
            state.handle_key(ProgrammerKey::Operator(operator));
            enter(&mut state, &[1, 1, 0, 0]);
            state.handle_key(ProgrammerKey::Equals);
            assert_eq!(state.value(), expected);
        }
    }

    #[test]
    fn shifts_bits_in_both_directions() {
        let mut left = ProgrammerState::default();
        enter(&mut left, &[3]);
        left.handle_key(ProgrammerKey::Operator(ProgrammerOperator::ShiftLeft));
        left.handle_key(ProgrammerKey::Digit(2));
        left.handle_key(ProgrammerKey::Equals);
        assert_eq!(left.value(), 12);

        let mut right = ProgrammerState::default();
        enter(&mut right, &[1, 2]);
        right.handle_key(ProgrammerKey::Operator(ProgrammerOperator::ShiftRight));
        right.handle_key(ProgrammerKey::Digit(2));
        right.handle_key(ProgrammerKey::Equals);
        assert_eq!(right.value(), 3);
    }

    #[test]
    fn complements_values_within_selected_word_size() {
        let mut state = ProgrammerState::default();
        state.set_word_size(WordSize::Bits8);
        state.handle_key(ProgrammerKey::Digit(1));
        state.handle_key(ProgrammerKey::Not);
        assert_eq!(state.value(), 0b1111_1110);

        state.handle_key(ProgrammerKey::TwosComplement);
        assert_eq!(state.value(), 2);
    }

    #[test]
    fn distinguishes_complement_operations_in_the_expression() {
        let mut state = ProgrammerState::default();
        state.set_word_size(WordSize::Bits8);
        state.handle_key(ProgrammerKey::Digit(1));

        state.handle_key(ProgrammerKey::OnesComplement);
        assert_eq!(state.value(), 254);
        assert_eq!(state.expression(), "1's complement 1");

        state.handle_key(ProgrammerKey::TwosComplement);
        assert_eq!(state.value(), 2);
        assert_eq!(state.expression(), "2's complement 254");
    }

    #[test]
    fn formats_the_selected_word_as_signed_twos_complement() {
        let mut state = ProgrammerState::default();
        state.set_word_size(WordSize::Bits8);
        state.handle_key(ProgrammerKey::Digit(5));
        assert_eq!(state.signed_decimal_conversion(), "5");

        state.handle_key(ProgrammerKey::TwosComplement);
        assert_eq!(state.value(), 251);
        assert_eq!(state.signed_decimal_conversion(), "-5");
    }

    #[test]
    fn toggles_only_bits_available_in_the_word() {
        let mut state = ProgrammerState::default();
        state.set_word_size(WordSize::Bits8);
        state.toggle_bit(7);
        state.toggle_bit(8);
        assert_eq!(state.value(), 128);
        assert!(state.bit_is_set(7));
        assert!(!state.bit_is_available(8));
    }

    #[test]
    fn smaller_word_sizes_mask_values_and_wrap_arithmetic() {
        let mut state = ProgrammerState::default();
        state.handle_key(ProgrammerKey::SetBase(NumberBase::Hexadecimal));
        enter(&mut state, &[15, 15]);
        state.set_word_size(WordSize::Bits8);
        state.handle_key(ProgrammerKey::Operator(ProgrammerOperator::Add));
        state.handle_key(ProgrammerKey::Digit(1));
        state.handle_key(ProgrammerKey::Equals);
        assert_eq!(state.value(), 0);
    }

    #[test]
    fn reports_division_by_zero() {
        let mut state = ProgrammerState::default();
        state.handle_key(ProgrammerKey::Digit(8));
        state.handle_key(ProgrammerKey::Operator(ProgrammerOperator::Divide));
        state.handle_key(ProgrammerKey::Digit(0));
        state.handle_key(ProgrammerKey::Equals);
        assert_eq!(state.display(), "Cannot divide by zero");
        assert!(state.has_error());
    }
}
