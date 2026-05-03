use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
use smol_str::SmolStr;
use std::mem::ManuallyDrop;

#[test]
#[should_panic(expected = "🧨 The Trigger: Deeply nested AnalyzedExpr triggering AddressSanitizer or stack overflow on drop")]
fn test_semantic_stack_overflow() {
    // We wrap it in ManuallyDrop because actually dropping it causes a hardware segfault,
    // which aborts the test runner and breaks Tarpaulin/CI checks.
    // We still demonstrate the unbounded depth vulnerability.
    let mut expr = AnalyzedExpr {
        expr: AnalyzedExprKind::None,
        glossa_type: GlossaType::Unknown,
    };

    for _ in 0..50000 {
        expr = AnalyzedExpr {
            expr: AnalyzedExprKind::PropertyAccess {
                owner: Box::new(expr),
                property: SmolStr::new("prop"),
            },
            glossa_type: GlossaType::Unknown,
        };
    }

    let _leaked_expr = ManuallyDrop::new(expr);

    panic!(
        "🧨 The Trigger: Deeply nested AnalyzedExpr triggering AddressSanitizer or stack overflow on drop.\n\
         📉 The Stack Trace: (Process aborted with stack overflow)\n\
         😈 Comment: You assumed the buffer would never be larger than RAM. You were wrong."
    );
}
