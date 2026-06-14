1. Execute `cat << 'EOF' > tests/havoc_codegen_stack_overflow.rs` to write the test file with the subprocess wrapper removed:
```rust
#![allow(missing_docs)]
use glossa::codegen::generate_rust;
use glossa::morphology::BinaryOp;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};

/// 👺 Havoc: Stack Overflow in Codegen (Direct)
///
/// If a deeply nested expression manages to bypass the parser limits or is
/// constructed programmatically, generating Rust code for it will immediately crash
/// the thread with a stack overflow.
#[test]
fn havoc_codegen_stack_overflow_direct() {
    let depth = 50_000;
    let mut expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(1),
        glossa_type: GlossaType::Number,
    };
    for _ in 0..depth {
        expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                left: Box::new(expr),
                op: BinaryOp::Add,
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
            },
            glossa_type: GlossaType::Number,
        };
    }

    let stmt = AnalyzedStatement::Expression(vec![expr]);
    let scope = Scope::new();
    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope,
    };

    // 💥 DETONATE
    println!(
        "Generating rust code for deep expression (depth {})...",
        depth
    );
    let _ = generate_rust(&program);
}
```
2. Execute `cat << 'EOF' > tests/havoc_semantic_stack_overflow.rs` to write the test file with the subprocess wrapper removed:
```rust
#![allow(missing_docs)]
use glossa::morphology::BinaryOp;
use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

/// 👺 Havoc: Stack Overflow in AnalyzedExpr Clone and Drop
///
/// Warden meticulously secured `Drop`, `Clone`, and `PartialEq`
/// using `stacker::maybe_grow` on the parser's AST (`Expr`), but
/// left the Semantic AST (`AnalyzedExpr` and `AnalyzedStatement`) completely exposed
/// to stack overflows via the derived `#[derive(Clone)]` and implicit `Drop`.
///
/// If a deeply nested expression manages to bypass the parser limits or is
/// constructed programmatically, dropping or cloning it will immediately crash
/// the thread with a stack overflow.
#[test]
fn havoc_semantic_clone_drop_stack_overflow() {
    let depth = 50_000;
    let mut expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(1),
        glossa_type: GlossaType::Number,
    };
    for _ in 0..depth {
        expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                left: Box::new(expr),
                op: BinaryOp::Add,
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
            },
            glossa_type: GlossaType::Number,
        };
    }

    // 💥 DETONATE
    println!("Cloning deep expression (depth {})...", depth);
    let expr2 = expr.clone();

    println!("Dropping cloned expression...");
    drop(expr2);

    println!("Dropping original expression...");
    drop(expr);
}
```
3. Run the full test suite using `cargo build && cargo test --features nova --lib && cargo test --features nova -p glossa --test '*' -- --skip havoc` to verify no happy-path regressions occurred.
4. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
5. Call `request_code_review` and then `submit` with the exact text:
Title: `👺 Havoc: Stack Overflow in AST Clone/Drop and Codegen`
Description:
```
🧨 **The Trigger:** A deeply nested programmatic `AnalyzedExpr` bypasses parser limits and overflows the stack because `AnalyzedExpr` uses a derived `Clone`/`Drop` missing `stacker::maybe_grow`, and `codegen::generate_rust` lacks stack growth checks.

📉 **The Stack Trace:**
thread 'havoc_codegen_stack_overflow_direct' has overflowed its stack
fatal runtime error: stack overflow, aborting

thread 'havoc_semantic_clone_drop_stack_overflow' has overflowed its stack
fatal runtime error: stack overflow, aborting

🧪 **Reproduction:** Run `cargo test --test havoc_codegen_stack_overflow` or `cargo test --test havoc_semantic_stack_overflow`.

😈 **Comment:** Warden locked the front door (parser AST) but left the back door (semantic AST and codegen) completely wide open. If it works, you failed.
```
