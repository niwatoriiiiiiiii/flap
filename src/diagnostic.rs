use crate::lexer::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub line_content: String,
    pub pointer_padding: usize,
    pub pointer_len: usize,
}

pub fn get_line_col(code: &str, char_index: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, c) in code.chars().enumerate() {
        if i == char_index {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub fn create_diagnostic(msg: &str, code: &str, span: Span) -> Diagnostic {
    let (line, col) = get_line_col(code, span.start);

    // Find the line content
    let lines: Vec<&str> = code.split('\n').collect();
    let line_content = if line > 0 && line <= lines.len() {
        lines[line - 1].to_string()
    } else {
        String::new()
    };

    let pointer_padding = col - 1;
    let pointer_len = (span.end - span.start).max(1);

    Diagnostic {
        message: msg.to_string(),
        line,
        col,
        line_content,
        pointer_padding,
        pointer_len,
    }
}
