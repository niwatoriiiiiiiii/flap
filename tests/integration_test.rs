use flap::interpreter::Interpreter;
use flap::lexer::lex;

#[test]
fn test_simple_arithmetic() {
    let code = "10,20+";
    let tokens = lex(code);
    let mut interpreter = Interpreter::new();
    interpreter.eval(&tokens);
    assert_eq!(*interpreter.stack(), vec![30]);
}

#[test]
fn test_stack_ops() {
    let code = "1,2:"; // 1, 2, 2
    let tokens = lex(code);
    let mut interpreter = Interpreter::new();
    interpreter.eval(&tokens);
    assert_eq!(*interpreter.stack(), vec![1, 2, 2]);
}

#[test]
fn test_loop_logic() {
    // 1 to 5 sum: 1, 2, 3, 4, 5 -> 15
    // Code: 0 5 [ : 1 - \ @ + \ ] ;
    // Let's use simpler loop test: 5 [ 1 - ] -> 5, 4, 3, 2, 1, 0
    let code = "5[:1-]";
    let tokens = lex(code);
    let mut interpreter = Interpreter::new();
    interpreter.eval(&tokens);
    // 5 (dup->5) 1 - -> 4. Loop checks 4.
    // 4 (dup->4) 1 - -> 3.
    // ...
    // 1 (dup->1) 1 - -> 0. Loop checks 0. Exit.
    // Stack should contain: 5, 4, 3, 2, 1, 0
    assert_eq!(*interpreter.stack(), vec![5, 4, 3, 2, 1, 0]);
}
