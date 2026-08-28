//! egui widgets, layout, and visual styling for the calculator.

use eframe::egui::{self, Align, Button, Color32, Layout, RichText, Vec2};

use crate::calculator::{AngleMode, MathematicalConstant, Operator, UnaryOperator};

use super::formatting::{display_expression, display_size};
use super::input::{
    Key, calculator_mode_toggle_requested, copy_result_requested, keyboard_keys,
    programmer_keyboard_keys,
};
use super::programmer::{NumberBase, ProgrammerKey, ProgrammerOperator, ProgrammerState, WordSize};
use super::state::CalculatorState;

const STANDARD_MIN_CALCULATOR_WIDTH: f32 = 300.0;
const STANDARD_MAX_CALCULATOR_WIDTH: f32 = 420.0;
const ADVANCED_MIN_CALCULATOR_WIDTH: f32 = 600.0;
const ADVANCED_MAX_CALCULATOR_WIDTH: f32 = 680.0;
const BUTTON_GAP: f32 = 9.0;
const BUTTON_HEIGHT: f32 = 52.0;
const BUTTON_ROW_GAP: f32 = 6.0;

pub(super) const STANDARD_MIN_WINDOW_SIZE: [f32; 2] = [340.0, 660.0];
pub(super) const STANDARD_WINDOW_SIZE: [f32; 2] = [380.0, 700.0];
const ADVANCED_MIN_WINDOW_SIZE: [f32; 2] = [640.0, 660.0];
const ADVANCED_WINDOW_SIZE: [f32; 2] = [720.0, 700.0];

/// Controls which set of calculator functions is visible and available.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum CalculatorMode {
    /// Shows only the controls expected from a basic calculator.
    #[default]
    Standard,
    /// Shows memory, grouping, constants, and scientific operations.
    Advanced,
    /// Shows integer bases, bits, and programming operations.
    Programmer,
}

impl CalculatorMode {
    /// Reports whether a calculator key is available in this mode.
    fn allows(self, key: Key) -> bool {
        match self {
            Self::Advanced => true,
            Self::Programmer => false,
            Self::Standard => matches!(
                key,
                Key::Number(_)
                    | Key::Decimal
                    | Key::Operator(
                        Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide
                    )
                    | Key::UnaryOperator(UnaryOperator::ToggleSign | UnaryOperator::Percent)
                    | Key::Equals
                    | Key::Backspace
                    | Key::Clear
            ),
        }
    }

    /// Returns the other available calculator mode.
    fn toggled(self) -> Self {
        match self {
            Self::Standard => Self::Advanced,
            Self::Advanced => Self::Programmer,
            Self::Programmer => Self::Standard,
        }
    }

    /// Returns the label shown in the mode selector.
    fn label(self) -> &'static str {
        match self {
            Self::Standard => "STANDARD",
            Self::Advanced => "ADVANCED",
            Self::Programmer => "PROGRAMMER",
        }
    }

    /// Returns the explanatory text shown when hovering over this mode.
    fn tooltip(self) -> &'static str {
        match self {
            Self::Standard => "Show basic calculator controls (F2)",
            Self::Advanced => "Show scientific calculator controls (F2)",
            Self::Programmer => "Show integer and bitwise controls (F2)",
        }
    }

    /// Returns the calculator-content width limits for this mode.
    fn calculator_widths(self) -> (f32, f32) {
        match self {
            Self::Standard => (STANDARD_MIN_CALCULATOR_WIDTH, STANDARD_MAX_CALCULATOR_WIDTH),
            Self::Advanced | Self::Programmer => {
                (ADVANCED_MIN_CALCULATOR_WIDTH, ADVANCED_MAX_CALCULATOR_WIDTH)
            }
        }
    }

    /// Returns the minimum and preferred native window sizes for this mode.
    fn window_sizes(self) -> ([f32; 2], [f32; 2]) {
        match self {
            Self::Standard => (STANDARD_MIN_WINDOW_SIZE, STANDARD_WINDOW_SIZE),
            Self::Advanced | Self::Programmer => (ADVANCED_MIN_WINDOW_SIZE, ADVANCED_WINDOW_SIZE),
        }
    }
}

/// Connects the calculator state to the native egui application lifecycle.
#[derive(Default)]
pub(super) struct CalculatorApp {
    state: CalculatorState,
    programmer: ProgrammerState,
    mode: CalculatorMode,
}

impl eframe::App for CalculatorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().set_visuals(egui::Visuals::dark());

        if copy_result_requested(ui.ctx()) {
            self.copy_result(ui.ctx());
        }
        if calculator_mode_toggle_requested(ui.ctx()) {
            self.set_mode(self.mode.toggled());
            self.resize_for_mode(ui.ctx());
        }

        if self.mode == CalculatorMode::Programmer {
            for key in programmer_keyboard_keys(ui.ctx()) {
                self.programmer.handle_key(key);
            }
        } else {
            for key in keyboard_keys(ui.ctx()) {
                if self.mode.allows(key) {
                    self.state.handle_key(key);
                }
            }
        }

        egui::Frame::default()
            .fill(Color32::from_rgb(17, 22, 29))
            .show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space(20.0);
                    self.show_calculator(ui);
                });
            });
    }
}

impl CalculatorApp {
    /// Changes the available controls and safely cancels an inaccessible open group.
    fn set_mode(&mut self, mode: CalculatorMode) {
        if mode != CalculatorMode::Advanced && self.state.has_open_parentheses() {
            self.state.handle_key(Key::Clear);
        }
        self.mode = mode;
    }

    /// Resizes the native window to the selected control layout.
    fn resize_for_mode(&self, context: &egui::Context) {
        let (minimum, size) = self.mode.window_sizes();
        context.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(
            minimum[0], minimum[1],
        )));
        context.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(
            size[0], size[1],
        )));
    }

    /// Copies the displayed result unless the calculator currently shows an error.
    fn copy_result(&self, context: &egui::Context) {
        match self.mode {
            CalculatorMode::Programmer if !self.programmer.has_error() => {
                context.copy_text(self.programmer.display());
            }
            CalculatorMode::Standard | CalculatorMode::Advanced if !self.state.has_error() => {
                context.copy_text(self.state.display().to_owned());
            }
            _ => {}
        }
    }

    /// Renders the title, angle-mode status, and copy action.
    fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Calculator")
                    .color(Color32::from_rgb(244, 248, 251))
                    .size(20.0)
                    .strong(),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let copy = ui
                    .add_enabled(
                        match self.mode {
                            CalculatorMode::Programmer => !self.programmer.has_error(),
                            _ => !self.state.has_error(),
                        },
                        Button::new(
                            RichText::new("COPY")
                                .color(Color32::from_rgb(220, 232, 242))
                                .size(12.0),
                        )
                        .fill(Color32::from_rgb(49, 61, 74))
                        .corner_radius(6.0),
                    )
                    .on_hover_text("Copy result (Ctrl+C / Cmd+C)");
                if copy.clicked() {
                    self.copy_result(ui.ctx());
                }

                if self.mode == CalculatorMode::Advanced && self.state.has_memory() {
                    ui.label(
                        RichText::new("M")
                            .color(Color32::from_rgb(126, 214, 168))
                            .size(12.0)
                            .strong(),
                    )
                    .on_hover_text("Calculator memory contains a value");
                }

                if self.mode == CalculatorMode::Advanced {
                    ui.label(
                        RichText::new(format!("ANGLE: {}", self.state.angle_mode().label()))
                            .color(Color32::from_rgb(115, 183, 235))
                            .size(12.0)
                            .strong(),
                    )
                    .on_hover_text("Active angle mode (M to switch)");
                }
            });
        });
    }

    /// Renders the switch between the basic and expanded control sets.
    fn show_mode_switch(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("MODE")
                    .color(Color32::from_rgb(143, 164, 183))
                    .size(12.0)
                    .strong(),
            );

            for mode in [
                CalculatorMode::Standard,
                CalculatorMode::Advanced,
                CalculatorMode::Programmer,
            ] {
                if ui
                    .selectable_label(self.mode == mode, mode.label())
                    .on_hover_text(mode.tooltip())
                    .clicked()
                {
                    self.set_mode(mode);
                    self.resize_for_mode(ui.ctx());
                }
            }
        });
    }

    /// Renders the calculator container and all of its sections.
    fn show_calculator(&mut self, ui: &mut egui::Ui) {
        let (minimum_width, maximum_width) = self.mode.calculator_widths();
        let calculator_width = (ui.available_width() - 40.0).clamp(minimum_width, maximum_width);

        egui::Frame::default()
            .fill(Color32::from_rgb(35, 43, 53))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(58, 70, 84)))
            .corner_radius(14.0)
            .inner_margin(18.0)
            .show(ui, |ui| {
                ui.set_width(calculator_width);
                self.show_header(ui);
                ui.add_space(8.0);
                self.show_mode_switch(ui);
                ui.add_space(12.0);
                if self.mode == CalculatorMode::Programmer {
                    self.show_programmer_display(ui);
                    ui.add_space(8.0);
                    self.show_programmer_settings(ui);
                    ui.add_space(6.0);
                    self.show_programmer_conversions(ui);
                    ui.add_space(6.0);
                    self.show_programmer_bits(ui);
                    ui.add_space(8.0);
                    self.show_programmer_buttons(ui);
                } else {
                    self.show_display(ui);
                    ui.add_space(14.0);
                    self.show_buttons(ui);
                }
            });
    }

    /// Renders the current expression and result display.
    fn show_display(&self, ui: &mut egui::Ui) {
        show_result_display(
            ui,
            DisplayContent {
                expression: self.state.expression(),
                value: self.state.display(),
                has_error: self.state.has_error(),
            },
            DisplayStyle {
                minimum_height: 90.0,
                expression_size: 14.0,
                value_size: display_size(self.state.display()),
                value_spacing: 8.0,
                monospace: false,
            },
        );
    }

    /// Renders the integer expression and active-base value in programmer mode.
    fn show_programmer_display(&self, ui: &mut egui::Ui) {
        let display = self.programmer.display();
        show_result_display(
            ui,
            DisplayContent {
                expression: self.programmer.expression(),
                value: &display,
                has_error: self.programmer.has_error(),
            },
            DisplayStyle {
                minimum_height: 70.0,
                expression_size: 13.0,
                value_size: programmer_display_size(&display),
                value_spacing: 4.0,
                monospace: true,
            },
        );
    }

    /// Renders base and word-size selectors for programmer mode.
    fn show_programmer_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("BASE").size(11.0).strong());
            for base in [
                NumberBase::Binary,
                NumberBase::Octal,
                NumberBase::Decimal,
                NumberBase::Hexadecimal,
            ] {
                if ui
                    .selectable_label(self.programmer.base() == base, base.label())
                    .on_hover_text(format!("Use base {}", base.label()))
                    .clicked()
                {
                    self.programmer.handle_key(ProgrammerKey::SetBase(base));
                }
            }

            ui.separator();
            ui.label(RichText::new("WORD").size(11.0).strong());
            for word_size in [
                WordSize::Bits8,
                WordSize::Bits16,
                WordSize::Bits32,
                WordSize::Bits64,
            ] {
                if ui
                    .selectable_label(self.programmer.word_size() == word_size, word_size.label())
                    .on_hover_text(format!("Use {} integers", word_size.label()))
                    .clicked()
                {
                    self.programmer.set_word_size(word_size);
                }
            }
        });
    }

    /// Renders the current value simultaneously in all supported bases.
    fn show_programmer_conversions(&mut self, ui: &mut egui::Ui) {
        egui::Frame::default()
            .fill(Color32::from_rgb(25, 32, 40))
            .corner_radius(8.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::Grid::new("programmer_conversions")
                    .num_columns(2)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
                        for base in [
                            NumberBase::Hexadecimal,
                            NumberBase::Decimal,
                            NumberBase::Octal,
                            NumberBase::Binary,
                        ] {
                            if ui
                                .selectable_label(self.programmer.base() == base, base.label())
                                .on_hover_text(format!("Switch input to {}", base.label()))
                                .clicked()
                            {
                                self.programmer.handle_key(ProgrammerKey::SetBase(base));
                            }
                            let conversion = if base == NumberBase::Decimal {
                                let unsigned = self.programmer.conversion(base);
                                let signed = self.programmer.signed_decimal_conversion();
                                if signed.starts_with('-') {
                                    format!("{unsigned}  (signed: {signed})")
                                } else {
                                    unsigned
                                }
                            } else {
                                self.programmer.conversion(base)
                            };
                            ui.label(
                                RichText::new(conversion)
                                    .color(Color32::from_rgb(218, 228, 237))
                                    .size(12.0)
                                    .monospace(),
                            );
                            ui.end_row();
                        }
                    });
            });
    }

    /// Renders a directly editable 64-bit representation of the current value.
    fn show_programmer_bits(&mut self, ui: &mut egui::Ui) {
        egui::Frame::default()
            .fill(Color32::from_rgb(25, 32, 40))
            .corner_radius(8.0)
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let gap = 2.0;
                let nibble_gap = 8.0;
                let bit_width = (ui.available_width() - gap * 15.0 - nibble_gap * 3.0) / 16.0;

                for row in 0..4 {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        let highest = 63 - row * 16;
                        for offset in 0..16 {
                            if offset > 0 && offset % 4 == 0 {
                                ui.add_space(nibble_gap);
                            }
                            let bit = highest - offset;
                            let available = self.programmer.bit_is_available(bit);
                            let set = self.programmer.bit_is_set(bit);
                            let button = Button::new(
                                RichText::new(if set { "1" } else { "0" })
                                    .monospace()
                                    .size(12.0),
                            )
                            .fill(if set {
                                Color32::from_rgb(48, 112, 160)
                            } else {
                                Color32::from_rgb(38, 48, 59)
                            })
                            .corner_radius(3.0);
                            if ui
                                .add_enabled_ui(available, |ui| {
                                    ui.add_sized([bit_width, 18.0], button)
                                })
                                .inner
                                .on_hover_text(format!("Toggle bit {bit}"))
                                .clicked()
                            {
                                self.programmer.toggle_bit(bit);
                            }
                        }
                    });
                }
            });
    }

    /// Renders the hexadecimal keypad and bitwise operation controls.
    fn show_programmer_buttons(&mut self, ui: &mut egui::Ui) {
        let rows = [
            [
                ProgrammerKey::Digit(10),
                ProgrammerKey::Digit(11),
                ProgrammerKey::Digit(12),
                ProgrammerKey::Digit(13),
                ProgrammerKey::Digit(7),
                ProgrammerKey::Digit(8),
                ProgrammerKey::Digit(9),
                ProgrammerKey::Operator(ProgrammerOperator::Divide),
            ],
            [
                ProgrammerKey::Digit(14),
                ProgrammerKey::Digit(15),
                ProgrammerKey::Operator(ProgrammerOperator::Modulo),
                ProgrammerKey::Not,
                ProgrammerKey::Digit(4),
                ProgrammerKey::Digit(5),
                ProgrammerKey::Digit(6),
                ProgrammerKey::Operator(ProgrammerOperator::Multiply),
            ],
            [
                ProgrammerKey::Operator(ProgrammerOperator::And),
                ProgrammerKey::Operator(ProgrammerOperator::Or),
                ProgrammerKey::Operator(ProgrammerOperator::Xor),
                ProgrammerKey::OnesComplement,
                ProgrammerKey::Digit(1),
                ProgrammerKey::Digit(2),
                ProgrammerKey::Digit(3),
                ProgrammerKey::Operator(ProgrammerOperator::Subtract),
            ],
            [
                ProgrammerKey::Operator(ProgrammerOperator::ShiftLeft),
                ProgrammerKey::Operator(ProgrammerOperator::ShiftRight),
                ProgrammerKey::TwosComplement,
                ProgrammerKey::Clear,
                ProgrammerKey::Digit(0),
                ProgrammerKey::Backspace,
                ProgrammerKey::Equals,
                ProgrammerKey::Operator(ProgrammerOperator::Add),
            ],
        ];

        for row in rows {
            self.show_programmer_button_row(ui, &row);
        }
    }

    /// Renders one programmer-mode keypad row.
    fn show_programmer_button_row(&mut self, ui: &mut egui::Ui, keys: &[ProgrammerKey]) {
        let gaps_width = BUTTON_GAP * keys.len().saturating_sub(1) as f32;
        let button_width = (ui.available_width() - gaps_width) / keys.len() as f32;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(BUTTON_GAP);
            for key in keys {
                let enabled = match key {
                    ProgrammerKey::Digit(digit) => self.programmer.base().accepts(*digit),
                    _ => true,
                };

                let active = match key {
                    ProgrammerKey::Operator(operator) => {
                        self.programmer.is_active_operator(*operator)
                    }
                    _ => false,
                };

                let tooltip = match key {
                    ProgrammerKey::Digit(digit) if !enabled => format!(
                        "Digit {} is not valid in {} (base {})",
                        programmer_digit_label(*digit),
                        self.programmer.base().label(),
                        match self.programmer.base() {
                            NumberBase::Binary => 2,
                            NumberBase::Octal => 8,
                            NumberBase::Decimal => 10,
                            NumberBase::Hexadecimal => 16,
                        }
                    ),
                    _ => programmer_tooltip(*key),
                };

                let response = ui
                    .add_enabled_ui(enabled, |ui| {
                        ui.add_sized([button_width, 40.0], programmer_button_for(*key, active))
                    })
                    .inner
                    .on_hover_text(tooltip);

                if response.clicked() {
                    self.programmer.handle_key(*key);
                }
            }
        });
        ui.add_space(4.0);
    }

    /// Renders all calculator button rows.
    fn show_buttons(&mut self, ui: &mut egui::Ui) {
        if self.mode == CalculatorMode::Advanced {
            self.show_advanced_button_rows(ui);
        } else {
            self.show_button_row(
                ui,
                &[
                    Key::UnaryOperator(UnaryOperator::ToggleSign),
                    Key::UnaryOperator(UnaryOperator::Percent),
                    Key::Clear,
                    Key::Backspace,
                ],
            );
            self.show_number_button_rows(ui);
        }
    }

    /// Renders controls available only in advanced mode.
    fn show_advanced_button_rows(&mut self, ui: &mut egui::Ui) {
        self.show_button_row(
            ui,
            &[
                Key::MemoryClear,
                Key::MemoryRecall,
                Key::MemoryAdd,
                Key::MemorySubtract,
                Key::OpenParenthesis,
                Key::CloseParenthesis,
                Key::Clear,
                Key::Backspace,
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::UnaryOperator(UnaryOperator::Sine),
                Key::UnaryOperator(UnaryOperator::Cosine),
                Key::UnaryOperator(UnaryOperator::Tangent),
                Key::UnaryOperator(UnaryOperator::HyperbolicSine),
                Key::UnaryOperator(UnaryOperator::HyperbolicCosine),
                Key::UnaryOperator(UnaryOperator::HyperbolicTangent),
                Key::UnaryOperator(UnaryOperator::AbsoluteValue),
                Key::ToggleAngleMode,
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::UnaryOperator(UnaryOperator::LogarithmBase10),
                Key::UnaryOperator(UnaryOperator::NaturalLogarithm),
                Key::UnaryOperator(UnaryOperator::Exponential),
                Key::Operator(Operator::Power),
                Key::Number('7'),
                Key::Number('8'),
                Key::Number('9'),
                Key::Operator(Operator::Divide),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::UnaryOperator(UnaryOperator::Reciprocal),
                Key::UnaryOperator(UnaryOperator::Factorial),
                Key::UnaryOperator(UnaryOperator::SquareRoot),
                Key::UnaryOperator(UnaryOperator::Square),
                Key::Number('4'),
                Key::Number('5'),
                Key::Number('6'),
                Key::Operator(Operator::Multiply),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::UnaryOperator(UnaryOperator::Floor),
                Key::UnaryOperator(UnaryOperator::Ceiling),
                Key::Operator(Operator::Modulo),
                Key::Operator(Operator::ScientificNotation),
                Key::Number('1'),
                Key::Number('2'),
                Key::Number('3'),
                Key::Operator(Operator::Subtract),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Constant(MathematicalConstant::Pi),
                Key::Constant(MathematicalConstant::Euler),
                Key::UnaryOperator(UnaryOperator::ToggleSign),
                Key::UnaryOperator(UnaryOperator::Percent),
                Key::Number('0'),
                Key::Decimal,
                Key::Equals,
                Key::Operator(Operator::Add),
            ],
        );
    }

    /// Renders the numeric keypad shared by both calculator modes.
    fn show_number_button_rows(&mut self, ui: &mut egui::Ui) {
        self.show_button_row(
            ui,
            &[
                Key::Number('7'),
                Key::Number('8'),
                Key::Number('9'),
                Key::Operator(Operator::Divide),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('4'),
                Key::Number('5'),
                Key::Number('6'),
                Key::Operator(Operator::Multiply),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('1'),
                Key::Number('2'),
                Key::Number('3'),
                Key::Operator(Operator::Subtract),
            ],
        );
        self.show_button_row(
            ui,
            &[
                Key::Number('0'),
                Key::Decimal,
                Key::Equals,
                Key::Operator(Operator::Add),
            ],
        );
    }

    /// Renders one row of buttons and forwards clicks to the state.
    fn show_button_row(&mut self, ui: &mut egui::Ui, keys: &[Key]) {
        let gaps_width = BUTTON_GAP * keys.len().saturating_sub(1) as f32;
        let button_width = (ui.available_width() - gaps_width) / keys.len() as f32;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(BUTTON_GAP);

            for key in keys {
                let response = ui
                    .add_sized(
                        [button_width, BUTTON_HEIGHT],
                        button_for(
                            *key,
                            self.state.is_active_operator(*key),
                            self.state.angle_mode(),
                        ),
                    )
                    .on_hover_text(tooltip_for(*key));
                if response.clicked() {
                    self.state.handle_key(*key);
                }
            }
        });
        ui.add_space(BUTTON_ROW_GAP);
    }
}

/// Dynamic text rendered by a calculator result display.
struct DisplayContent<'a> {
    expression: &'a str,
    value: &'a str,
    has_error: bool,
}

/// Visual differences between regular and programmer result displays.
struct DisplayStyle {
    minimum_height: f32,
    expression_size: f32,
    value_size: f32,
    value_spacing: f32,
    monospace: bool,
}

/// Renders the shared expression-and-result display frame.
fn show_result_display(ui: &mut egui::Ui, content: DisplayContent<'_>, style: DisplayStyle) {
    egui::Frame::default()
        .fill(Color32::from_rgb(11, 17, 24))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(61, 76, 90)))
        .corner_radius(10.0)
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(style.minimum_height);
            ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                let mut expression = RichText::new(display_expression(content.expression))
                    .color(Color32::from_rgb(143, 164, 183))
                    .size(style.expression_size);
                if style.monospace {
                    expression = expression.monospace();
                }
                ui.label(expression);

                let value_color = if content.has_error {
                    Color32::from_rgb(255, 189, 189)
                } else {
                    Color32::from_rgb(247, 251, 255)
                };
                let mut value = RichText::new(content.value)
                    .color(value_color)
                    .size(style.value_size)
                    .strong();
                if style.monospace {
                    value = value.monospace();
                }

                ui.add_space(style.value_spacing);
                ui.label(value);
            });
        });
}

/// Creates a styled button for a calculator action.
fn button_for(key: Key, active: bool, angle_mode: AngleMode) -> Button<'static> {
    let (label, color, text_color) = match key {
        Key::Number(number) => (
            number.to_string(),
            key_color(),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Decimal => (
            ".".to_owned(),
            key_color(),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Operator(operator) => (
            match operator {
                Operator::Power => "x^y",
                Operator::ScientificNotation => "x×10^y",
                _ => operator.symbol(),
            }
            .to_owned(),
            if active {
                Color32::from_rgb(244, 248, 251)
            } else {
                Color32::from_rgb(238, 135, 65)
            },
            if active {
                Color32::from_rgb(28, 36, 45)
            } else {
                Color32::WHITE
            },
        ),
        Key::UnaryOperator(operator) => (
            match operator {
                UnaryOperator::ToggleSign => "±",
                UnaryOperator::Percent => "%",
                UnaryOperator::SquareRoot => "√",
                UnaryOperator::Square => "x²",
                UnaryOperator::Sine => "sin",
                UnaryOperator::Cosine => "cos",
                UnaryOperator::Tangent => "tan",
                UnaryOperator::LogarithmBase10 => "log₁₀",
                UnaryOperator::NaturalLogarithm => "ln",
                UnaryOperator::Exponential => "e^x",
                UnaryOperator::Reciprocal => "1/x",
                UnaryOperator::Factorial => "x!",
                UnaryOperator::HyperbolicSine => "sinh",
                UnaryOperator::HyperbolicCosine => "cosh",
                UnaryOperator::HyperbolicTangent => "tanh",
                UnaryOperator::AbsoluteValue => "|x|",
                UnaryOperator::Floor => "floor",
                UnaryOperator::Ceiling => "ceil",
            }
            .to_owned(),
            Color32::from_rgb(67, 80, 95),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Constant(constant) => (
            constant.symbol().to_owned(),
            Color32::from_rgb(48, 100, 145),
            Color32::WHITE,
        ),
        Key::MemoryClear | Key::MemoryRecall | Key::MemoryAdd | Key::MemorySubtract => (
            match key {
                Key::MemoryClear => "MC",
                Key::MemoryRecall => "MR",
                Key::MemoryAdd => "M+",
                Key::MemorySubtract => "M−",
                _ => unreachable!(),
            }
            .to_owned(),
            Color32::from_rgb(55, 91, 112),
            Color32::from_rgb(226, 241, 249),
        ),
        Key::OpenParenthesis | Key::CloseParenthesis => (
            if key == Key::OpenParenthesis {
                "(".to_owned()
            } else {
                ")".to_owned()
            },
            Color32::from_rgb(67, 80, 95),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Equals => (
            "=".to_owned(),
            Color32::from_rgb(42, 146, 103),
            Color32::WHITE,
        ),
        Key::Backspace => (
            "DEL".to_owned(),
            Color32::from_rgb(67, 80, 95),
            Color32::from_rgb(246, 249, 252),
        ),
        Key::Clear => (
            "AC".to_owned(),
            Color32::from_rgb(150, 70, 78),
            Color32::from_rgb(255, 245, 245),
        ),
        Key::ToggleAngleMode => (
            angle_mode.label().to_owned(),
            Color32::from_rgb(48, 100, 145),
            Color32::WHITE,
        ),
    };

    styled_button(label, color, text_color, 20.0, 10.0)
}

/// Returns a short description and keyboard shortcut for a calculator key.
fn tooltip_for(key: Key) -> String {
    match key {
        Key::Number(number) => format!("Enter {number} ({number})"),
        Key::Decimal => "Decimal point (. or ,)".to_owned(),
        Key::Operator(operator) => match operator {
            Operator::Add => "Addition (+)",
            Operator::Subtract => "Subtraction (-)",
            Operator::Multiply => "Multiplication (*)",
            Operator::Divide => "Division (/)",
            Operator::Power => "Power (^)",
            Operator::Modulo => "Modulo division (D)",
            Operator::ScientificNotation => "Scientific exponent x×10^y (J)",
        }
        .to_owned(),
        Key::UnaryOperator(operator) => match operator {
            UnaryOperator::ToggleSign => "Toggle sign (N)",
            UnaryOperator::Percent => "Percentage (%)",
            UnaryOperator::SquareRoot => "Square root (R)",
            UnaryOperator::Square => "Square (S)",
            UnaryOperator::Sine => "Sine (I)",
            UnaryOperator::Cosine => "Cosine (C)",
            UnaryOperator::Tangent => "Tangent (T)",
            UnaryOperator::LogarithmBase10 => "Base-10 logarithm (O)",
            UnaryOperator::NaturalLogarithm => "Natural logarithm (L)",
            UnaryOperator::Exponential => "Exponential e^x (E)",
            UnaryOperator::Reciprocal => "Reciprocal (V)",
            UnaryOperator::Factorial => "Factorial (F)",
            UnaryOperator::HyperbolicSine => "Hyperbolic sine (H)",
            UnaryOperator::HyperbolicCosine => "Hyperbolic cosine (U)",
            UnaryOperator::HyperbolicTangent => "Hyperbolic tangent (Y)",
            UnaryOperator::AbsoluteValue => "Absolute value (A)",
            UnaryOperator::Floor => "Round down / floor (G)",
            UnaryOperator::Ceiling => "Round up / ceil (B)",
        }
        .to_owned(),
        Key::Constant(constant) => match constant {
            MathematicalConstant::Pi => "Enter pi (P)",
            MathematicalConstant::Euler => "Enter Euler's number (K)",
        }
        .to_owned(),
        Key::MemoryClear => "Clear memory (Ctrl+L)".to_owned(),
        Key::MemoryRecall => "Recall memory (Ctrl+R)".to_owned(),
        Key::MemoryAdd => "Add display to memory (Ctrl+P)".to_owned(),
        Key::MemorySubtract => "Subtract display from memory (Ctrl+Q)".to_owned(),
        Key::OpenParenthesis => "Open parenthesis — key: (".to_owned(),
        Key::CloseParenthesis => "Close parenthesis — key: )".to_owned(),
        Key::Equals => "Calculate result (Enter or =)".to_owned(),
        Key::Backspace => "Delete last digit (Backspace)".to_owned(),
        Key::Clear => "Clear calculator (Escape or Delete)".to_owned(),
        Key::ToggleAngleMode => "Switch angle mode (M)".to_owned(),
    }
}

/// Creates a styled programmer-mode button.
fn programmer_button_for(key: ProgrammerKey, active: bool) -> Button<'static> {
    let (label, color) = match key {
        ProgrammerKey::Digit(digit) => (programmer_digit_label(digit).to_string(), key_color()),
        ProgrammerKey::Operator(operator) => (
            operator.symbol().to_owned(),
            if active {
                Color32::from_rgb(244, 248, 251)
            } else {
                Color32::from_rgb(238, 135, 65)
            },
        ),
        ProgrammerKey::Equals => ("=".to_owned(), Color32::from_rgb(42, 146, 103)),
        ProgrammerKey::Clear => ("AC".to_owned(), Color32::from_rgb(150, 70, 78)),
        ProgrammerKey::Backspace => ("DEL".to_owned(), Color32::from_rgb(67, 80, 95)),
        ProgrammerKey::Not => ("NOT".to_owned(), Color32::from_rgb(55, 91, 112)),
        ProgrammerKey::OnesComplement => ("1's C".to_owned(), Color32::from_rgb(55, 91, 112)),
        ProgrammerKey::TwosComplement => ("2's C".to_owned(), Color32::from_rgb(55, 91, 112)),
        ProgrammerKey::SetBase(base) => (base.label().to_owned(), Color32::from_rgb(48, 100, 145)),
    };
    let text_color = if active {
        Color32::from_rgb(28, 36, 45)
    } else {
        Color32::WHITE
    };

    styled_button(label, color, text_color, 16.0, 8.0)
}

/// Applies the visual treatment shared by all calculator keypad buttons.
fn styled_button(
    label: String,
    color: Color32,
    text_color: Color32,
    font_size: f32,
    corner_radius: f32,
) -> Button<'static> {
    Button::new(
        RichText::new(label)
            .color(text_color)
            .size(font_size)
            .strong(),
    )
    .fill(color)
    .stroke(egui::Stroke::new(
        1.0,
        Color32::from_rgba_unmultiplied(255, 255, 255, 24),
    ))
    .corner_radius(corner_radius)
}

/// Returns the description and shortcut for a programmer-mode key.
fn programmer_tooltip(key: ProgrammerKey) -> String {
    match key {
        ProgrammerKey::Digit(digit) => format!("Enter {}", programmer_digit_label(digit)),
        ProgrammerKey::Operator(operator) => match operator {
            ProgrammerOperator::Add => "Wrapping addition (+)",
            ProgrammerOperator::Subtract => "Wrapping subtraction (-)",
            ProgrammerOperator::Multiply => "Wrapping multiplication (*)",
            ProgrammerOperator::Divide => "Integer division (/)",
            ProgrammerOperator::Modulo => "Integer remainder (%)",
            ProgrammerOperator::And => "Bitwise AND (&)",
            ProgrammerOperator::Or => "Bitwise OR (|)",
            ProgrammerOperator::Xor => "Bitwise XOR (^)",
            ProgrammerOperator::ShiftLeft => "Shift left (<)",
            ProgrammerOperator::ShiftRight => "Shift right (>)",
        }
        .to_owned(),
        ProgrammerKey::Equals => "Calculate result (Enter or =)".to_owned(),
        ProgrammerKey::Clear => "Clear programmer calculation (Escape)".to_owned(),
        ProgrammerKey::Backspace => "Delete last digit (Backspace)".to_owned(),
        ProgrammerKey::Not => "Bitwise NOT (~): flip every bit in the selected word".to_owned(),
        ProgrammerKey::OnesComplement => {
            "One's complement: flip every bit (same result as NOT)".to_owned()
        }
        ProgrammerKey::TwosComplement => {
            "Two's complement: flip every bit, then add 1 (integer negation)".to_owned()
        }
        ProgrammerKey::SetBase(base) => format!("Use {} input", base.label()),
    }
}

/// Formats a programmer digit using the conventional upper-case hexadecimal notation.
fn programmer_digit_label(digit: u8) -> char {
    char::from_digit(u32::from(digit), 16)
        .unwrap_or('0')
        .to_ascii_uppercase()
}

/// Returns the background color used by number and decimal buttons.
fn key_color() -> Color32 {
    Color32::from_rgb(49, 61, 74)
}

/// Chooses a compact font size for long binary programmer values.
fn programmer_display_size(display: &str) -> f32 {
    if display.len() > 60 {
        13.0
    } else if display.len() > 32 {
        18.0
    } else {
        28.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_mode_allows_only_basic_calculator_keys() {
        let mode = CalculatorMode::Standard;

        assert!(mode.allows(Key::Number('7')));
        assert!(mode.allows(Key::Operator(Operator::Add)));
        assert!(mode.allows(Key::Operator(Operator::Multiply)));
        assert!(mode.allows(Key::UnaryOperator(UnaryOperator::Percent)));
        assert!(mode.allows(Key::Clear));

        assert!(!mode.allows(Key::Operator(Operator::Power)));
        assert!(!mode.allows(Key::Operator(Operator::Modulo)));
        assert!(!mode.allows(Key::Operator(Operator::ScientificNotation)));
        assert!(!mode.allows(Key::UnaryOperator(UnaryOperator::Sine)));
        assert!(!mode.allows(Key::UnaryOperator(UnaryOperator::HyperbolicSine)));
        assert!(!mode.allows(Key::Constant(MathematicalConstant::Pi)));
        assert!(!mode.allows(Key::MemoryRecall));
        assert!(!mode.allows(Key::OpenParenthesis));
        assert!(!mode.allows(Key::ToggleAngleMode));
    }

    #[test]
    fn advanced_mode_allows_scientific_calculator_keys() {
        let mode = CalculatorMode::Advanced;

        assert!(mode.allows(Key::Operator(Operator::Power)));
        assert!(mode.allows(Key::Operator(Operator::Modulo)));
        assert!(mode.allows(Key::Operator(Operator::ScientificNotation)));
        assert!(mode.allows(Key::UnaryOperator(UnaryOperator::Sine)));
        assert!(mode.allows(Key::UnaryOperator(UnaryOperator::HyperbolicSine)));
        assert!(mode.allows(Key::Constant(MathematicalConstant::Pi)));
        assert!(mode.allows(Key::MemoryRecall));
        assert!(mode.allows(Key::OpenParenthesis));
        assert!(mode.allows(Key::ToggleAngleMode));
    }

    #[test]
    fn programmer_mode_uses_its_own_input_actions() {
        let mode = CalculatorMode::Programmer;

        assert!(!mode.allows(Key::Number('7')));
        assert!(!mode.allows(Key::Operator(Operator::Add)));
        assert!(!mode.allows(Key::UnaryOperator(UnaryOperator::Sine)));
    }

    #[test]
    fn mode_shortcut_cycles_through_all_three_views() {
        assert_eq!(CalculatorMode::Standard.toggled(), CalculatorMode::Advanced);
        assert_eq!(
            CalculatorMode::Advanced.toggled(),
            CalculatorMode::Programmer
        );
        assert_eq!(
            CalculatorMode::Programmer.toggled(),
            CalculatorMode::Standard
        );
    }

    #[test]
    fn calculator_starts_in_standard_mode() {
        let app = CalculatorApp::default();
        assert_eq!(app.mode, CalculatorMode::Standard);
    }

    #[test]
    fn switching_to_standard_cancels_an_open_group() {
        let mut app = CalculatorApp {
            mode: CalculatorMode::Advanced,
            ..CalculatorApp::default()
        };
        app.state.handle_key(Key::OpenParenthesis);
        assert!(app.state.has_open_parentheses());

        app.set_mode(CalculatorMode::Standard);

        assert_eq!(app.mode, CalculatorMode::Standard);
        assert!(!app.state.has_open_parentheses());
        assert_eq!(app.state.display(), "0");
    }

    #[test]
    fn programmer_display_font_shrinks_for_binary_values() {
        assert_eq!(programmer_display_size("1010"), 28.0);
        assert_eq!(programmer_display_size(&"1".repeat(40)), 18.0);
        assert_eq!(programmer_display_size(&"1".repeat(64)), 13.0);
    }
}
