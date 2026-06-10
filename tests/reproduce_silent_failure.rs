#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_undefined_struct_type_error() {
    let code = "
    εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.
    χρήστης νέον Χρήστος «Σωκράτης» ἔστω.
    ";

    let ast = parse(code).unwrap();
    let result = analyze_program(&ast);

    match result {
        Ok(_) => panic!(
            "Expected compilation error for undefined type 'Χρήστος', but compilation succeeded."
        ),
        Err(e) => {
            let msg = e.to_string();
            // Expected error message should contain the unknown name
            assert!(
                msg.contains("Χρήστος")
                    || msg.contains("undefined")
                    || msg.contains("Οὐκ οἶδα τὸ")
                    || msg.contains("Άγνωστον")
            );
        }
    }
}
