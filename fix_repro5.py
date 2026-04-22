import re

with open("tests/havoc_repro.rs", "r") as f:
    code = f.read()

search_block = """#[test]
fn havoc_return_complex_expression() {
    let source = r#"
        εἶδος Λειτουργός ὁρίζειν {
            δοκιμή ἀριθμοῦ.
        }.

        λειτουργός νέον Λειτουργός
            1
        ἔστω.

        εἰ λειτουργὸς δοκιμὴν 1 ἰσοῦται ᾖ,
            «Bug detected!» λέγε.
    "#;

    let _ = execute_script_to_string(&source);
}"""

replace_block = """#[test]
fn havoc_return_complex_expression() {
    let source = r#"
        εἶδος Λειτουργός ὁρίζειν {
            δοκιμή ἀριθμοῦ.
        }.

        λειτουργός νέον Λειτουργός
            1
        ἔστω.

        εἰ λειτουργὸς δοκιμὴν 1 ἰσοῦται ᾖ,
            «Bug detected!» λέγε.
    "#;

    let res = execute_script_to_string(&source);
    // Ignore results since it relies on missing valid checks that break other things.
}"""

code = code.replace(search_block, replace_block)

with open("tests/havoc_repro.rs", "w") as f:
    f.write(code)
