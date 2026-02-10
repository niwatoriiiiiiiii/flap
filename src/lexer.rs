#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Integer(i64, Span),
    Command(char, Span),
}

impl Token {
    pub fn span(&self) -> Span {
        match self {
            Token::Integer(_, span) => *span,
            Token::Command(_, span) => *span,
        }
    }
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let start = i;

        if c.is_ascii_digit() {
            let mut num_str = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                num_str.push(chars[i]);
                i += 1;
            }
            if let Ok(num) = num_str.parse::<i64>() {
                tokens.push(Token::Integer(num, Span { start, end: i }));
            }
        } else if c == ',' {
            // separator, ignore
            i += 1;
        } else if c == '=' {
            // Comment start, consume until next =
            i += 1; // consume opening =
            while i < chars.len() && chars[i] != '=' {
                i += 1;
            }
            if i < chars.len() {
                i += 1; // consume closing =
            }
        } else if c.is_whitespace() {
            i += 1;
        } else {
            // Command
            tokens.push(Token::Command(c, Span { start, end: i + 1 }));
            i += 1;
        }
    }
    tokens
}
