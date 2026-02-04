use crate::lexer::Token;
use std::io::{self, Read, Write};

pub struct Interpreter {
    stack: Vec<i64>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn stack(&self) -> &Vec<i64> {
        &self.stack
    }

    pub fn eval(&mut self, tokens: &[Token]) {
        let mut pc = 0;
        while pc < tokens.len() {
            match &tokens[pc] {
                Token::Integer(val) => self.stack.push(*val),
                Token::Command(cmd) => {
                    self.execute_command(*cmd, tokens, &mut pc);
                }
            }
            pc += 1;
        }
    }

    fn execute_command(&mut self, cmd: char, tokens: &[Token], pc: &mut usize) {
        match cmd {
            // Arithmetic
            '+' => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                self.stack.push(a + b);
            }
            '-' => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                self.stack.push(a - b);
            }
            '*' => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                self.stack.push(a * b);
            }
            '/' => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                if b != 0 {
                    self.stack.push(a / b);
                } else {
                    // Division by zero: pushing 0 or handling error? Spec didn't strictly specify panic behavior after revert.
                    // Assuming safe behavior: push 0 or keep stack. Let's push 0 for now.
                    self.stack.push(0);
                }
            }
            '%' => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                if b != 0 {
                    self.stack.push(a % b);
                } else {
                    self.stack.push(0);
                }
            }

            // Stack Ops
            ':' => {
                // Dup
                if let Some(&val) = self.stack.last() {
                    self.stack.push(val);
                }
            }
            ';' => {
                // Pop
                self.stack.pop();
            }
            'x' => {
                // Swap
                if self.stack.len() >= 2 {
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                }
            }
            '@' => {
                // Rot (a, b, c -> b, c, a)
                if self.stack.len() >= 3 {
                    let c = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(b);
                    self.stack.push(c);
                    self.stack.push(a);
                }
            }
            'R' => {
                // Reverse entire stack
                self.stack.reverse();
            }

            // IO
            'r' => {
                // Read Num
                let val = self.read_number();
                self.stack.push(val);
            }
            'T' => {
                // Read Text - read entire line and push each char as code
                let text = self.read_text();
                for ch in text.chars() {
                    self.stack.push(ch as i64);
                }
            }
            'p' => {
                // Print Num
                if let Some(val) = self.stack.pop() {
                    print!("{}", val);
                    io::stdout().flush().unwrap();
                }
            }
            'P' => {
                // Print Char
                if let Some(val) = self.stack.pop() {
                    let c = (val as u8) as char;
                    print!("{}", c);
                    io::stdout().flush().unwrap();
                }
            }

            // Control Flow
            '[' => {
                // While Start
                // Check if top is 0 (without popping)
                let val = self.stack.last().copied().unwrap_or(0);
                if val == 0 {
                    // Jump to matching ]
                    if let Some(target) = self.find_matching(tokens, *pc, '[', ']', 1) {
                        *pc = target;
                    }
                }
            }
            ']' => {
                // While End
                // Jump back to matching [
                if let Some(target) = self.find_matching(tokens, *pc, ']', '[', -1) {
                    *pc = target - 1; // -1 because loop will increment pc
                }
            }
            '(' => {
                // If Start
                // Check if top is 0 (without popping yet?) Spec says "Pop after execution".
                // Logic: If 0, jump to matching ). Else, continue.
                let val = self.stack.last().copied().unwrap_or(0);
                if val == 0 {
                    if let Some(target) = self.find_matching(tokens, *pc, '(', ')', 1) {
                        *pc = target - 1;
                        // And pop the 0 (per spec "Pop after execution", meaning pop happens at end of block or skipped block?)
                        // If we skip, we land at ')'. ')' will handle the pop?
                    }
                }
            }
            ')' => {
                // If End
                // Always pop the condition value
                self.stack.pop();
            }

            _ => {}
        }
    }

    fn read_number(&self) -> i64 {
        // Simple synchronous read from stdin.
        // In a real golf runner, we might need buffering.
        // For now, read a word.
        let mut word = String::new();
        let mut buffer = [0; 1];
        let mut started = false;

        // Skip whitespace then read digits
        loop {
            match io::stdin().read_exact(&mut buffer) {
                Ok(_) => {
                    let c = buffer[0] as char;
                    if c.is_ascii_whitespace() {
                        if started {
                            break;
                        } // End of word
                    } else {
                        started = true;
                        word.push(c);
                    }
                }
                Err(_) => break, // EOF or error
            }
        }
        word.parse().unwrap_or(0)
    }

    fn read_text(&self) -> String {
        // Read entire line from stdin
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap_or(0);
        // Remove trailing newline
        line.trim_end().to_string()
    }

    fn find_matching(
        &self,
        tokens: &[Token],
        start: usize,
        open: char,
        close: char,
        dir: i32,
    ) -> Option<usize> {
        let mut depth = 1; // Already sitting on one bracket
        let mut i = start as i32 + dir;
        let len = tokens.len() as i32;

        while i >= 0 && i < len {
            match &tokens[i as usize] {
                Token::Command(c) if *c == open => depth += 1,
                Token::Command(c) if *c == close => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i as usize);
                    }
                }
                _ => {}
            }
            i += dir;
        }
        None
    }
}
