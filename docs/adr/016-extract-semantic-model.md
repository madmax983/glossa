# 16. Extract Semantic Model

Date: 2026-03-13
Status: Accepted

## Context

During a structural audit by Atlas, a circular dependency was identified within the Semantic Analysis module ("Breaking The Knot: Semantic Types and Models"). Specifically, `src/semantic/types.rs` depended on `AnalyzedStatement` (located in `src/semantic/mod.rs`) for method bodies, while `mod.rs` depended on `GlossaType` (located in `types.rs`).

This tangle meant the Type System relied directly on the AST it was intended to type, blurring the boundaries between Data (Model), Types (Type System), and Logic (Analysis). This tight coupling made testing and future architectural refactoring fragile.

## Decision

We have decided to strictly separate the structural models from the type system and the analysis logic:

1. Created `src/semantic/model.rs` to serve as a pure data container for all AST nodes (`AnalyzedStatement`, `AnalyzedExpr`) and Semantic Models (`TraitDef`, `TraitImpl`).
2. Moved `TraitDef` and associated structures from `types.rs` to `model.rs`.
3. Moved AST nodes from `mod.rs` to `model.rs`.
4. Refactored `mod.rs` to re-export `model` contents for backward compatibility.
5. Removed legacy `SemanticAnalyzer` code from `mod.rs` that was duplicating logic unnecessarily.

## Consequences

### Positive
- **Acyclic Graph**: The circular dependency is broken. The dependency graph is now strictly `model.rs` -> `types.rs`, with `types.rs` functioning as a leaf-level module containing no dependencies on the AST.
- **Separation of Concerns**: Data (AST), Type System (`GlossaType`), and Analysis Logic are now strictly isolated in their respective modules.
- **Maintainability**: Future changes to AST structure will not automatically necessitate changes to or trigger rebuilds of the core Type System utilities.

### Negative
- **Indirection**: Moving types around required updating imports and re-exports, adding slight indirection, though `mod.rs` continues to act as a convenient facade.