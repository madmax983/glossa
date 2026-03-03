#![no_main]

use libfuzzer_sys::fuzz_target;
use glossa::ast::{Expr, Word};

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    // We want to construct something that triggers the unsafe block in Drop
    // For example, an ArrayLiteral containing an ArrayLiteral

    let string_val = String::from("test string to detect double free");

    // Create an array that holds a StringLiteral.
    let leaf = Expr::StringLiteral(string_val);

    let mut expr = leaf;

    // Build some nesting based on data
    for &byte in data.iter().take(10) {
        expr = match byte % 5 {
            0 => Expr::Phrase(vec![expr]),
            1 => Expr::ArrayLiteral(vec![expr]),
            2 => Expr::IndexAccess {
                array: Box::new(expr),
                index: Box::new(Expr::NumberLiteral(1))
            },
            3 => Expr::PropertyAccess {
                owner: Box::new(expr),
                property: Box::new(Expr::Word(Word::new("prop")))
            },
            _ => Expr::Binding {
                name: Word::new("name"),
                value: Box::new(expr)
            }
        };
    }

    // Drop happens here!
});
