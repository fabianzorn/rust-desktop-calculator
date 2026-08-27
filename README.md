# Calculator

A small desktop calculator written in Rust. The user interface is built with
[`eframe`](https://github.com/emilk/egui/tree/main/crates/eframe) and
[`egui`](https://github.com/emilk/egui).

The calculator currently supports:

- Addition
- Subtraction
- Multiplication
- Division
- Sign toggle (`±`)
- Percent calculations (`%`)
- Square roots (`√`)
- Squaring (`x²`)
- Sine calculations (`sin`)
- Cosine calculations (`cos`)
- Tangent calculations (`tan`)
- Base-10 logarithms (`log₁₀`)
- Natural logarithms (`ln`)
- Exponential calculations (`e^x`)
- Arbitrary powers (`x^y`)
- Reciprocals (`1/x`)
- Factorials (`x!`)
- Mathematical constants (`π`, `e`)
- Switchable degree and radian angle modes (`DEG`/`RAD`)
- Decimal numbers
- Clearing and deleting input
- Chained calculations
- Keyboard controls
- Tooltips showing available keyboard shortcuts
- Copying the current result
- A persistent active angle-mode indicator
- Manual input limited to 18 characters
- Scientific notation for very large and very small results
- Error handling for invalid mathematical domains and out-of-range results

## Keyboard controls

The calculator can be operated with the mouse or keyboard:

| Key | Action |
| --- | --- |
| `0`-`9` | Enter digits |
| `.` or `,` | Enter a decimal point |
| `+`, `-`, `*`, `/`, `^` | Select a binary operation |
| `Enter` or `=` | Calculate the result |
| `Backspace` | Delete the last digit |
| `Escape` or `Delete` | Clear the calculator |
| `%` | Convert the current value to a percentage |
| `R` | Calculate the square root |
| `S` | Square the current value |
| `N` | Toggle the sign |
| `I` | Calculate the sine in the selected angle mode |
| `C` | Calculate the cosine in the selected angle mode |
| `T` | Calculate the tangent in the selected angle mode |
| `M` | Switch between degree and radian mode |
| `O` | Calculate the base-10 logarithm |
| `L` | Calculate the natural logarithm |
| `E` | Calculate e raised to the current value |
| `V` | Calculate the reciprocal |
| `F` | Calculate the factorial |
| `P` | Enter π |
| `K` | Enter Euler's number e |
| `Ctrl+C` (`Cmd+C` on macOS) | Copy the current result |

## Requirements

- [Rustup](https://rustup.rs/)
- A C/C++ build toolchain
- Linux system libraries required by `eframe`

The required Rust version and components are defined in
`rust-toolchain.toml`. Rustup automatically selects Rust 1.98.0 and installs
`rustfmt` and `clippy` when commands are run inside the project directory.

On Ubuntu or Debian, install the required native dependency with:

```bash
sudo apt-get update
sudo apt-get install libxkbcommon-dev
```

## Run the application

Start the calculator in development mode:

```bash
cargo run
```

## Build

Create a debug build:

```bash
cargo build --locked
```

Create an optimized release build:

```bash
cargo build --locked --release
```

The resulting executable is located at `target/release/calculator`.

## Test

Run all tests for all targets and enabled features:

```bash
cargo test --locked --all-targets --all-features
```

## Format

Format the source code:

```bash
cargo fmt --all
```

Check formatting without modifying files:

```bash
cargo fmt --all -- --check
```

## Clippy

Run Clippy for all targets and features. Warnings are treated as errors, just
as they are in CI:

```bash
cargo clippy --locked --all-targets --all-features -- -D warnings
```

## Run all CI checks locally

Before pushing changes, run:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo build --locked --release
```

## Project structure

```text
.
├── .github/workflows/ci.yml  # GitHub Actions pipeline
├── src/
│   ├── calculator.rs         # Calculation logic and related tests
│   ├── desktop_ui.rs         # Desktop UI module entry point and window setup
│   ├── desktop_ui/
│   │   ├── formatting.rs     # Number, expression, and display formatting
│   │   ├── input.rs          # Keyboard input and calculator key mapping
│   │   ├── state.rs          # Calculator state, interactions, and related tests
│   │   └── view.rs           # egui layout, controls, and styling
│   └── main.rs               # Application entry point
├── Cargo.toml                # Package metadata and dependencies
└── rust-toolchain.toml       # Rust version and components
```

## Continuous integration

The GitHub Actions workflow runs formatting checks, Clippy, tests, and a
release build for pushes and pull requests targeting the `main` or `develop`
branches. It can also be started manually from the **Actions** tab on GitHub.
