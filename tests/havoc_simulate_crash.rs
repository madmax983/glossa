#[cfg(feature = "nova")]
use glossa::experimental::interpreter::Interpreter;
#[cfg(feature = "nova")]
use glossa::parser::parse;
#[cfg(feature = "nova")]
use glossa::semantic::analyze_program;

#[cfg(feature = "nova")]
#[test]
fn test_havoc_simulate_overflow_panic() {
    // 🧨 The Trigger: "Input string with length 0xFFFFFF caused buffer overflow."
    // In our case, the trigger is an integer overflow during semantic interpretation.
    let source = "ξ 9223372036854775807 ἔστω. ξ ἄθροισμα 1 λέγε."; // i64::MAX + 1
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();

    let mut interpreter = Interpreter::new();

    // 💥 DETONATE: This will panic when evaluating `i64::MAX + 1` in the interpreter!
    // "thread 'test_havoc_simulate_overflow_panic' panicked at attempt to add with overflow"
    interpreter.run(&analyzed).unwrap();
}
