use crate::ast::Expr;
use crate::semantic::expressions::feed_expr_to_assembler_with_context;
use crate::semantic::{Assembler, DisambiguationContext};

#[test]
fn test_property_access_on_accusative_owner() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();

    let owner = Expr::Word(crate::ast::Word::new("λόγον"));
    let property = Expr::Word(crate::ast::Word::new("μῆκος"));
    let expr = Expr::PropertyAccess {
        owner: Box::new(owner),
        property: Box::new(property),
    };

    feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx).unwrap();
    let stmt = asm.finalize().unwrap();

    // EXPECTATION: The Property Access should be preserved as a nested phrase
    // This allows extract_value to handle it correctly later
    assert!(
        !stmt.nested_phrases.is_empty(),
        "Should have nested phrase containing property access"
    );

    // Verify content of nested phrase
    let phrase = &stmt.nested_phrases[0];
    assert_eq!(phrase.len(), 1);
    match &phrase[0] {
        Expr::PropertyAccess { .. } => {} // Success
        _ => panic!(
            "Nested phrase should contain PropertyAccess, found {:?}",
            phrase[0]
        ),
    }
}

#[test]
fn test_nested_property_access() {
    let mut asm = Assembler::new();
    let mut ctx = DisambiguationContext::new();

    let inner_phrase = Expr::Phrase(vec![
        Expr::NumberLiteral(1),
        Expr::Word(crate::ast::Word::new("+")),
        Expr::NumberLiteral(2),
    ]);
    let nested_phrase = Expr::Phrase(vec![inner_phrase]);

    let property = Expr::Word(crate::ast::Word::new("μῆκος"));
    let expr = Expr::PropertyAccess {
        owner: Box::new(nested_phrase),
        property: Box::new(property),
    };

    feed_expr_to_assembler_with_context(&mut asm, &expr, &mut ctx).unwrap();
    let stmt = asm.finalize().unwrap();

    // EXPECTATION: The Property Access should be preserved as a nested phrase
    assert!(
        !stmt.nested_phrases.is_empty(),
        "Should have nested phrase containing property access"
    );

    let phrase = &stmt.nested_phrases[0];
    assert_eq!(phrase.len(), 1);
    match &phrase[0] {
        Expr::PropertyAccess { .. } => {} // Success
        _ => panic!(
            "Nested phrase should contain PropertyAccess, found {:?}",
            phrase[0]
        ),
    }
}
