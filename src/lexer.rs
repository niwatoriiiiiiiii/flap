#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Integer(i64),
    Command(char),
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            let mut num_str = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num_str.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Ok(num) = num_str.parse::<i64>() {
                tokens.push(Token::Integer(num));
            }
        } else if c == ',' {
            // separator, ignore
            chars.next();
        } else if c == '=' {
            // Comment start, consume until next =
            chars.next(); // consume opening =
            while let Some(&comment_char) = chars.peek() {
                chars.next(); // consume char
                if comment_char == '=' {
                    break;
                }
            }
        } else if c.is_whitespace() {
            chars.next();
        } else {
            // Command
            tokens.push(Token::Command(c));
            chars.next();
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_numbers() {
        let input = "10,20";
        let tokens = lex(input);
        assert_eq!(tokens, vec![Token::Integer(10), Token::Integer(20)]);
    }

    #[test]
    fn test_lex_commands() {
        let input = "1,10+p";
        let tokens = lex(input);
        assert_eq!(
            tokens,
            vec![
                Token::Integer(1),
                Token::Integer(10),
                Token::Command('+'),
                Token::Command('p')
            ]
        );
    }
}
