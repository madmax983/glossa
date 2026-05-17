## 2025-02-23 - Added edge-case AST depth and trait code-generation coverage
**Learning:** Found that edge cases in trait definitions (specifically those without a return type and body) were not tested in `src/codegen.rs`, hiding potential `unreachable!` or ignored arms. Additionally, deeply nested ASTs that exceed recursion limits lacked coverage in `src/semantic/validation.rs`.
**Action:** Created synthetic tests explicitly for AST depth > 250 and trait definitions missing bodies and returns, bumping total coverage up and verifying limit checks.
