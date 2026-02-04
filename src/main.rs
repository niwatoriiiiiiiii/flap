use flap::{interpreter, lexer};
use std::env;

use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    let input = &args[1];
    let code = if Path::new(input).exists() {
        fs::read_to_string(input).unwrap_or_else(|_| {
            eprintln!("Error: Could not read file '{}'", input);
            std::process::exit(1);
        })
    } else {
        input.to_string()
    };

    let tokens = lexer::lex(&code);
    let mut interpreter = interpreter::Interpreter::new();
    interpreter.eval(&tokens);
}
