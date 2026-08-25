# Calculator

A small desktop calculator written in Rust. The user interface is built with
[`eframe`](https://github.com/emilk/egui/tree/main/crates/eframe) and
[`egui`](https://github.com/emilk/egui).

The calculator currently supports:

- Addition
- Subtraction
- Multiplication
- Division
- Decimal numbers
- Clearing and deleting input
- Chained calculations
- Division-by-zero error handling

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
│   ├── desktop_ui.rs         # Desktop UI, input handling, and UI tests
│   └── main.rs               # Application entry point
├── Cargo.toml                # Package metadata and dependencies
└── rust-toolchain.toml       # Rust version and components
```

## Continuous integration

The GitHub Actions workflow runs formatting checks, Clippy, tests, and a
release build for pushes and pull requests targeting the `main` or `develop`
branches. It can also be started manually from the **Actions** tab on GitHub.
