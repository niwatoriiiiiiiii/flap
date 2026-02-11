# Flap

[English](./README.md) | [日本語](./README.ja.md)

Flap is a minimal stack-based esoteric programming language (Esolang) implemented in Rust.

## Features

- Minimalist command set
- Stack-based architecture
- Infinite loops and conditional branching
- Integers and ASCII character support

## Installation

Download the latest `flap_installer` for your platform from [Releases](https://github.com/niwatoriiiiiiiii/flap/releases) and run it.

Alternatively, if you have [Rust](https://www.rust-lang.org/) installed:

```bash
git clone https://github.com/niwatoriiiiiiiii/flap.git
cd flap
cargo install --path .
```

## Usage

Flap provides a simple CLI to manage your projects:

```bash
# Initialize a new project (creates src/main.flap and flap.toml)
flap init

# Run src/main.flap in the current directory
flap run

# Start interactive REPL mode
flap repl

# Update flap to the latest version
flap update

# Run a specific file
flap <file>
```

## Language Specification

The full language specification is available here:

- [English Specification](./language_spec_en.md)
- [Japanese Specification](./language_spec_jp.md)
