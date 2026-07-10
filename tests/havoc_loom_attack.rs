#![allow(missing_docs)]
use loom::thread;

// Loom intercepts thread creation.
#[test]
fn havoc_loom_parser_torture() {
    loom::model(|| {
        let input = "ξ 1 ἔστω. ξ λέγε.";
        let ast = glossa::parser::parse(input).unwrap();

        let t1 = thread::spawn(move || {
            let _ = glossa::semantic::analyze_program(&ast);
        });

        let ast2 = glossa::parser::parse(input).unwrap();
        let t2 = thread::spawn(move || {
            let _ = glossa::semantic::analyze_program(&ast2);
        });

        t1.join().unwrap();
        t2.join().unwrap();
    });
}
