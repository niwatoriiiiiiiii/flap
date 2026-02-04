# Flap

[English](./README.md) | [日本語](./README.ja.md)

Flap is a minimal stack-based esoteric programming language (Esolang) implemented in Rust.

## Features

- Minimalist command set
- Stack-based architecture
- Infinite loops and conditional branching
- Integers and ASCII character support

## Installation

Ensure you have [Rust](https://www.rust-lang.org/) installed, then:

```bash
git clone https://github.com/niwatoriiiiiiiii/flap.git
cd flap
cargo build --release
```

## Usage

Run a `.flap` file or provide code as a string:

```bash
# Run a file
cargo run -- examples/hello_flap.flap

# Run a code string directly
cargo run -- "10,20+p"
```

## Language Specification

The full language specification is available here:

- [English Specification](./language_spec_en.md)
- [Japanese Specification](./language_spec_jp.md)

## Examples

See the [examples/](./examples) directory for sample programs:

- `hello_flap.flap`: Classic "Hello, World!"
- `add.flap`: Addition demo
- `parity.flap`: Even/Odd checker using `if`
- `echo.flap`: Echo input
