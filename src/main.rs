use colored::*;
use flap::config::Config;
use flap::diagnostic::create_diagnostic;
use flap::interpreter::{ExecutionResult, Interpreter, RuntimeError};
use flap::lexer::{Token, lex};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::load();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "--version" => {
            println!("flap {}", VERSION);
        }
        "--help" => {
            print_help();
        }
        "repl" => {
            run_repl(&config);
        }
        "run" => {
            run_project(&config);
        }
        "init" => {
            init_project();
        }
        "update" => {
            update_flap();
        }
        arg => {
            run_file(arg, &config);
        }
    }
}

fn print_help() {
    println!("Flap Programming Language v{}", VERSION);
    println!();
    println!("Usage:");
    println!("  flap             Show version and help");
    println!("  flap --version   Show version");
    println!("  flap --help      Show help");
    println!("  flap repl        Start interactive mode");
    println!("  flap run         Run src/main.flap");
    println!("  flap init        Initialize project");
    println!("  flap update      Update flap to the latest version");
    println!("  flap <file>      Run specified file");
}

fn update_flap() {
    println!("Checking for updates...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("niwatoriiiiiiiii")
        .repo_name("flap")
        .bin_name("flap")
        .show_download_progress(true)
        .current_version(VERSION)
        .build();

    match status {
        Ok(updater) => match updater.update() {
            Ok(status) => {
                if status.updated() {
                    println!("Updated to version {}!", status.version());
                } else {
                    println!("Already up to date.");
                }
            }
            Err(e) => {
                println!("{} {}", "Update failed:".red(), e);
            }
        },
        Err(e) => {
            println!("{} {}", "Failed to configure updater:".red(), e);
        }
    }
}

fn init_project() {
    if Path::new("flap.toml").exists() {
        println!("{}", "Error: flap.toml already exists".red());
        return;
    }

    let toml_content = r#"# Flap Configuration
time_limit = 2000
stack_size = 131072
output_size = 1048576
time_check_interval = 1000
"#;

    if let Err(e) = fs::write("flap.toml", toml_content) {
        println!("{} {}", "Error creating flap.toml:".red(), e);
        return;
    }

    if !Path::new("src").exists() {
        if let Err(e) = fs::create_dir("src") {
            println!("{} {}", "Error creating src directory:".red(), e);
            return;
        }
    }

    let main_flap = "src/main.flap";
    if !Path::new(main_flap).exists() {
        let code = r#"
10, 20 + p
"#;
        if let Err(e) = fs::write(main_flap, code.trim_start()) {
            println!("{} {}", "Error creating src/main.flap:".red(), e);
            return;
        }
    }

    println!("{}", "Project initialized successfully.".green());
    println!("Run 'flap run' to start.");
}

fn run_project(config: &Config) {
    let path = "src/main.flap";
    if Path::new(path).exists() {
        run_file(path, config);
    } else {
        println!(
            "{} {}",
            "Error: src/main.flap not found.".red(),
            "(Run 'flap init' to create a new project)"
        );
    }
}

fn run_file(path: &str, config: &Config) {
    match fs::read_to_string(path) {
        Ok(code) => {
            let tokens = lex(&code);
            let stdin = io::stdin();
            let stdout = io::stdout();

            let mut interpreter = Interpreter::with_options(
                stdin.lock(),
                stdout,
                Duration::from_millis(config.time_limit),
                config.stack_size,
                config.output_size,
                config.time_check_interval,
            );

            let (res, _) = interpreter.eval(&tokens);

            match res {
                ExecutionResult::Ok => {}
                ExecutionResult::RuntimeError(e) => {
                    print_error(&e, &code, path, &tokens, interpreter.pc());
                    std::process::exit(1);
                }
                ExecutionResult::TimeLimitExceeded => {
                    println!("{}", "Error: Time Limit Exceeded".red());
                    std::process::exit(1);
                }
                ExecutionResult::MemoryLimitExceeded => {
                    println!("{}", "Error: Memory Limit Exceeded".red());
                    std::process::exit(1);
                }
                _ => {}
            }
        }
        Err(e) => {
            println!("{} {}", format!("Error reading file '{}':", path).red(), e);
        }
    }
}

fn run_repl(config: &Config) {
    println!("Flap REPL v{}", VERSION);
    println!("Type 'exit' to quit.");

    let mut input_buffer = String::new();
    let mut stack = Vec::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        input_buffer.clear();

        if io::stdin().read_line(&mut input_buffer).is_err() {
            println!();
            break;
        }

        let trimmed = input_buffer.trim();
        if trimmed == "exit" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        let tokens = lex(trimmed);

        // Use stdin for the interpreter input
        // Since we read line by line for the commands, stdin is available for the program input
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        let mut interpreter = Interpreter::with_options(
            stdin.lock(),
            &mut stdout,
            Duration::from_millis(config.time_limit),
            config.stack_size,
            config.output_size,
            config.time_check_interval,
        );

        interpreter.load_stack(stack.clone());

        let (res, _) = interpreter.eval(&tokens);

        match res {
            ExecutionResult::Ok => {
                stack = interpreter.stack().clone();
                // If the executed code produced output (via 'p'), we might want a newline?
                // But the user might have handled it.
                // We typically print the stack top if it's an expression.
                // User example shows clean output.
                // Let's print the last item if stack grew?
                // Or just always print last item?
                // "1 line input, 1 line output" was the original request.
                // Let's just print the top of the stack.
                if let Some(last) = stack.last() {
                    println!("{}", last);
                }
            }
            ExecutionResult::RuntimeError(e) => {
                print_error(&e, trimmed, "REPL", &tokens, interpreter.pc());
            }
            ExecutionResult::TimeLimitExceeded => {
                println!("{}", "Error: Time Limit Exceeded".red());
            }
            ExecutionResult::MemoryLimitExceeded => {
                println!("{}", "Error: Memory Limit Exceeded".red());
            }
            _ => {}
        }
    }
}

fn print_error(e: &RuntimeError, code: &str, filename: &str, tokens: &[Token], pc: usize) {
    if pc >= tokens.len() {
        println!("{} {}", "error:".red().bold(), e);
        return;
    }
    let span = tokens[pc].span();
    let diag = create_diagnostic(&e.to_string(), code, span);

    println!("{} {}", "error:".red().bold(), diag.message.bold());
    println!("  {} {}:{}:{}", "-->".blue(), filename, diag.line, diag.col);

    let line_num = diag.line.to_string();
    let pad = " ".repeat(line_num.len());

    println!(" {} {}", pad, "|".blue());
    println!(" {} {} {}", line_num.blue(), "|".blue(), diag.line_content);

    let pointer_line = format!(
        "{}{}",
        " ".repeat(diag.pointer_padding),
        "^".repeat(diag.pointer_len)
    );
    println!(" {} {} {}", pad, "|".blue(), pointer_line.red().bold());
}
