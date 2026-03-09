#[cfg(feature = "nova")]
#[test]
fn test_interpreter_types() {
    use glossa::tools::Interpreter;
    let interp = Interpreter::new();
    assert_eq!(interp.get_output(), "");
}
