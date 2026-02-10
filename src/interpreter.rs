use crate::lexer::Token;
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    StackUnderflow,
    DivisionByZero,
    IoError(String),
    InvalidInput,
    UnmatchedBracket,
    UnknownCommand(char),
    OutputLimitExceeded,
}

/// 実行結果
#[derive(Debug, PartialEq)]
pub enum ExecutionResult {
    Ok,
    Running, // To identify partial execution
    RuntimeError(RuntimeError),
    TimeLimitExceeded,
    MemoryLimitExceeded,
}

// 時間チェック間隔（ステップ数）- Default
pub const DEFAULT_TIME_CHECK_INTERVAL: u64 = 1_000;
// デフォルトの制限時間
pub const DEFAULT_TIME_LIMIT: Duration = Duration::from_millis(2000);
// 出力制限 (1 MiB)
pub const MAX_OUTPUT_SIZE: usize = 1024 * 1024;
// メモリ制限 (1 MiB = 1024 * 1024 bytes)
pub const MAX_STACK_SIZE: usize = 1024 * 1024 / 8;

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::StackUnderflow => write!(f, "Stack underflow"),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::IoError(msg) => write!(f, "I/O error: {}", msg),
            RuntimeError::InvalidInput => write!(f, "Invalid input"),
            RuntimeError::UnmatchedBracket => write!(f, "Unmatched bracket"),
            RuntimeError::UnknownCommand(c) => write!(f, "Unknown command: '{}'", c),
            RuntimeError::OutputLimitExceeded => write!(f, "Output limit exceeded (max 1MB)"),
        }
    }
}

impl std::fmt::Display for ExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionResult::Ok => write!(f, "Success"),
            ExecutionResult::Running => write!(f, "Running"),
            ExecutionResult::RuntimeError(e) => write!(f, "Runtime Error: {}", e),
            ExecutionResult::TimeLimitExceeded => write!(f, "Time Limit Exceeded (TLE)"),
            ExecutionResult::MemoryLimitExceeded => write!(f, "Memory Limit Exceeded (MLE)"),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub struct Interpreter<R: BufRead, W: Write> {
    stack: Vec<i64>,
    input: R,
    output: W,
    time_limit: Duration,
    max_stack_depth: usize,
    pc: usize,
    steps: u64,
    bytes_written: usize,
    start_time: Option<Instant>,
    max_stack_size: usize,
    max_output_size: usize,
    time_check_interval: u64,
}

// 実行結果とメモリ使用量を返すタプル
type EvalResult = (ExecutionResult, usize);

impl<R: BufRead, W: Write> Interpreter<R, W> {
    pub fn new(input: R, output: W) -> Self {
        Self {
            stack: Vec::new(),
            input,
            output,
            time_limit: DEFAULT_TIME_LIMIT,
            max_stack_depth: 0,
            pc: 0,
            steps: 0,
            bytes_written: 0,
            start_time: None,
            max_stack_size: MAX_STACK_SIZE,
            max_output_size: MAX_OUTPUT_SIZE,
            time_check_interval: DEFAULT_TIME_CHECK_INTERVAL,
        }
    }

    pub fn with_options(
        input: R,
        output: W,
        time_limit: Duration,
        max_stack_size: usize,
        max_output_size: usize,
        time_check_interval: u64,
    ) -> Self {
        Self {
            stack: Vec::new(),
            input,
            output,
            time_limit,
            max_stack_depth: 0,
            pc: 0,
            steps: 0,
            bytes_written: 0,
            start_time: None,
            max_stack_size,
            max_output_size,
            time_check_interval,
        }
    }

    pub fn stack(&self) -> &Vec<i64> {
        &self.stack
    }

    pub fn load_stack(&mut self, stack: Vec<i64>) {
        self.stack = stack;
        self.max_stack_depth = self.max_stack_depth.max(self.stack.len());
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn output(&self) -> &W {
        &self.output
    }

    // 安全にスタックにプッシュする（制限チェック付き）
    fn push_safe(&mut self, val: i64) -> Result<(), ()> {
        if self.stack.len() >= self.max_stack_size {
            return Err(());
        }
        self.stack.push(val);
        self.max_stack_depth = self.max_stack_depth.max(self.stack.len());
        Ok(())
    }

    pub fn step(&mut self, tokens: &[Token]) -> ExecutionResult {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        if self.pc >= tokens.len() {
            return ExecutionResult::Ok;
        }

        self.steps += 1;

        if self.steps % self.time_check_interval == 0 {
            if let Some(start) = self.start_time {
                if start.elapsed() > self.time_limit {
                    return ExecutionResult::TimeLimitExceeded;
                }
            }
        }

        match &tokens[self.pc] {
            Token::Integer(val, _) => {
                if self.push_safe(*val).is_err() {
                    return ExecutionResult::MemoryLimitExceeded;
                }
            }
            Token::Command(cmd, _) => {
                let result = match cmd {
                    'p' | 'P' => {
                        let mut wrapper = LimitWriter {
                            inner: &mut self.output,
                            written: self.bytes_written,
                            limit: self.max_output_size,
                        };
                        let exec_res = Self::execute_command_with_writer(
                            &mut self.stack,
                            &mut self.input,
                            *cmd,
                            tokens,
                            &mut self.pc,
                            &mut wrapper,
                        );
                        self.bytes_written = wrapper.written;
                        exec_res
                    }
                    _ => Self::execute_command_with_writer(
                        &mut self.stack,
                        &mut self.input,
                        *cmd,
                        tokens,
                        &mut self.pc,
                        &mut self.output,
                    ),
                };

                match result {
                    Ok(_) => {} // Continue
                    Err(e) => return ExecutionResult::RuntimeError(e),
                }

                if self.stack.len() > self.max_stack_size {
                    return ExecutionResult::MemoryLimitExceeded;
                }
                self.max_stack_depth = self.max_stack_depth.max(self.stack.len());
            }
        }

        self.pc += 1;
        if self.pc >= tokens.len() {
            ExecutionResult::Ok
        } else {
            ExecutionResult::Running
        }
    }

    pub fn eval(&mut self, tokens: &[Token]) -> EvalResult {
        loop {
            let res = self.step(tokens);
            match res {
                ExecutionResult::Running => continue,
                _ => return (res, self.max_stack_depth),
            }
        }
    }

    fn execute_command_with_writer<R2: BufRead, W2: Write>(
        stack: &mut Vec<i64>,
        input: &mut R2,
        cmd: char,
        tokens: &[Token],
        pc: &mut usize,
        writer: &mut W2,
    ) -> Result<(), RuntimeError> {
        match cmd {
            // Arithmetic
            '+' => {
                let b = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                let a = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                stack.push(a + b);
            }
            '-' => {
                let b = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                let a = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                stack.push(a - b);
            }
            '*' => {
                let b = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                let a = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                stack.push(a * b);
            }
            '/' => {
                let b = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                let a = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                if b != 0 {
                    stack.push(a / b);
                } else {
                    return Err(RuntimeError::DivisionByZero);
                }
            }
            '%' => {
                let b = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                let a = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                if b != 0 {
                    stack.push(a % b);
                } else {
                    return Err(RuntimeError::DivisionByZero);
                }
            }

            // Stack Ops
            ':' => {
                // Dup
                let val = *stack.last().ok_or(RuntimeError::StackUnderflow)?;
                stack.push(val);
            }
            ';' => {
                // Pop
                stack.pop().ok_or(RuntimeError::StackUnderflow)?;
            }
            'x' => {
                // Swap
                if stack.len() >= 2 {
                    let len = stack.len();
                    stack.swap(len - 1, len - 2);
                } else {
                    return Err(RuntimeError::StackUnderflow);
                }
            }
            '@' => {
                // Rot (a, b, c -> b, c, a)
                if stack.len() >= 3 {
                    let c = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(b);
                    stack.push(c);
                    stack.push(a);
                } else {
                    return Err(RuntimeError::StackUnderflow);
                }
            }
            'R' => {
                // Reverse entire stack
                stack.reverse();
            }

            // IO
            'r' => {
                // Read Num
                let val = Self::read_number_static(input)?;
                stack.push(val);
            }
            't' => {
                // Read Text - read one char
                let val = Self::read_byte_static(input)?;
                stack.push(val);
            }
            'p' => {
                // Print Num
                let val = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                write!(writer, "{}", val).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::Other
                        && e.to_string() == "Output limit exceeded"
                    {
                        RuntimeError::OutputLimitExceeded
                    } else {
                        RuntimeError::IoError(e.to_string())
                    }
                })?;
            }
            'P' => {
                // Print Char
                let val = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                let c = (val as u8) as char;
                write!(writer, "{}", c).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::Other
                        && e.to_string() == "Output limit exceeded"
                    {
                        RuntimeError::OutputLimitExceeded
                    } else {
                        RuntimeError::IoError(e.to_string())
                    }
                })?;
            }

            // Control Flow
            '[' => {
                // While Start
                // Check if top is 0 (without popping)
                let val = *stack.last().ok_or(RuntimeError::StackUnderflow)?;
                if val == 0 {
                    // Jump to matching ]
                    let target = Self::find_matching_static(tokens, *pc, '[', ']', 1)
                        .ok_or(RuntimeError::UnmatchedBracket)?;
                    *pc = target;
                }
            }
            ']' => {
                // While End
                // Jump back to matching [
                let target = Self::find_matching_static(tokens, *pc, ']', '[', -1)
                    .ok_or(RuntimeError::UnmatchedBracket)?;
                *pc = target - 1; // -1 because loop will increment pc
            }
            '(' => {
                // If Start
                // Pop the condition value immediately
                let val = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                if val == 0 {
                    // Jump to matching )
                    let target = Self::find_matching_static(tokens, *pc, '(', ')', 1)
                        .ok_or(RuntimeError::UnmatchedBracket)?;
                    *pc = target;
                }
            }
            ')' => {
                // If End
                // Do nothing (condition already popped at start)
            }

            _ => return Err(RuntimeError::UnknownCommand(cmd)),
        }
        Ok(())
    }

    fn read_number_static<R2: BufRead>(input: &mut R2) -> Result<i64, RuntimeError> {
        let mut word = String::new();
        let mut buffer = [0; 1];
        let mut started = false;

        loop {
            match input.read_exact(&mut buffer) {
                Ok(_) => {
                    let c = buffer[0] as char;
                    if c.is_ascii_whitespace() {
                        if started {
                            // If we hit \r, try to consume \n if present
                            if c == '\r' {
                                // Peek next byte
                                let buf = input
                                    .fill_buf()
                                    .map_err(|e| RuntimeError::IoError(e.to_string()))?;
                                if !buf.is_empty() && buf[0] == b'\n' {
                                    input.consume(1);
                                }
                            }
                            break;
                        }
                    } else {
                        started = true;
                        word.push(c);
                    }
                }
                Err(_) => break,
            }
        }
        if word.is_empty() {
            return Ok(0);
        }
        word.parse().map_err(|_| RuntimeError::InvalidInput)
    }

    fn read_byte_static<R2: BufRead>(input: &mut R2) -> Result<i64, RuntimeError> {
        let mut buf = [0; 1];
        match input.read(&mut buf) {
            Ok(1) => Ok(buf[0] as i64),
            Ok(0) => Ok(0),
            Err(e) => Err(RuntimeError::IoError(e.to_string())),
            _ => Ok(0),
        }
    }

    fn find_matching_static(
        tokens: &[Token],
        start: usize,
        open: char,
        close: char,
        dir: i32,
    ) -> Option<usize> {
        let mut depth = 1;
        let mut i = start as i32 + dir;
        let len = tokens.len() as i32;

        while i >= 0 && i < len {
            match &tokens[i as usize] {
                Token::Command(c, _) if *c == open => depth += 1,
                Token::Command(c, _) if *c == close => {
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

// 出力制限を監視するためのラッパー
struct LimitWriter<'a, W: Write> {
    inner: &'a mut W,
    written: usize,
    limit: usize,
}

impl<'a, W: Write> Write for LimitWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.written + buf.len() > self.limit {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Output limit exceeded",
            ));
        }
        let n = self.inner.write(buf)?;
        self.written += n;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
