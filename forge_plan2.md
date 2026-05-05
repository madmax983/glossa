# Refactor `test_dot_generator_coverage` in `src/tools/haruspex.rs`

## Problem (Smell)
The test `test_dot_generator_coverage` is 275 lines long. It manually builds an enormous `AnalyzedProgram` by pushing dozens of statement and expression variants into a `Vec`, then passes it to `HaruspexVisitor` and asserts the output. This is hard to read and modify.

## Solution
Break this large test into smaller, logically grouped tests:
1. `test_dot_generator_basic_statements` (Binding, Assignment, Print, Query, Expression)
2. `test_dot_generator_control_flow` (If, While, For, Match, Break, Continue)
3. `test_dot_generator_functions_types` (FunctionDef, Return, TypeDefinition, TraitDefinition, TraitImplementation)
4. `test_dot_generator_expressions` (all `AnalyzedExprKind` variants wrapped in an Expression statement)

Create a helper function to run the visitor on a set of statements and assert the output.

## Verification
- Run `cargo test` to ensure tests still pass.
- Run `cargo clippy` to ensure warnings are resolved.
