use flap::interpreter::Interpreter;
use flap::lexer::lex;

#[test]
fn test_comments() {
    let code = "10 = this is ignored = 20 +";
    let tokens = lex(code);
    let mut interpreter = Interpreter::new(std::io::empty(), Vec::new());
    interpreter.eval(&tokens);
    assert_eq!(*interpreter.stack(), vec![30]);
}

#[test]
fn test_swap_x() {
    let code = "1,2 x"; // push 1, 2, then swap -> 2, 1
    let tokens = lex(code);
    let mut interpreter = Interpreter::new(std::io::empty(), Vec::new());
    interpreter.eval(&tokens);
    assert_eq!(*interpreter.stack(), vec![2, 1]);
}

#[test]
fn test_reverse_r() {
    let code = "1,2,3 R";
    let tokens = lex(code);
    let mut interpreter = Interpreter::new(std::io::empty(), Vec::new());
    interpreter.eval(&tokens);
    assert_eq!(*interpreter.stack(), vec![3, 2, 1]);
}
