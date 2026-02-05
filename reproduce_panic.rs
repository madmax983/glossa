
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use glossa::codegen::rust::generate_rust;

fn main() {
    // A Greek word consisting ONLY of a combining diacritic (e.g. U+0301 combining acute accent)
    // The grammar allows GREEK_CHAR+, where GREEK_CHAR includes GREEK_COMBINING.
    // So "\u{0301}" is a valid greek_word.

    // Attempting to use it as a variable name:
    // "\u{0301} πέντε ἔστω." -> Let '´' be 5.

    let source = "\u{0301} πέντε ἔστω.";
    println!("Parsing source: {:?}", source);

    let ast = parse(source).expect("Failed to parse");
    println!("AST parsed successfully");

    let analyzed = analyze_program(&ast).expect("Failed to analyze");
    println!("Analysis successful");

    // This should panic inside generate_rust -> sanitize_name -> format_ident!
    let rust_code = generate_rust(&analyzed);
    println!("Generated Rust code: {}", rust_code);
}
