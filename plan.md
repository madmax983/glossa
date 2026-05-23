1. **Understand the Havoc Persona Goal:** Identify a weak point in the Glossa compiler that leads to a crash, panic, race condition, or deadlock, and write a failing "wreckage" test that detonates this weakness. Do *not* fix the bug. Submit the PR with the failing test.
2. **Current Finding:** `AnalyzedExpr` is a recursive enum (`AnalyzedExprKind::BinOp { left: Box<AnalyzedExpr>, ... }`). When a deeply nested `AnalyzedExpr` goes out of scope, the Rust compiler implicitly drops it, which walks the tree recursively. This causes a stack overflow (`SIGABRT`) on the main thread!
3. **Execution Plan:**
   - I have already verified the stack overflow using `tests/havoc_semantic_drop.rs`.
   - The test must fail `cargo test` because it detonates. (Wait, if `cargo test` fails, the test suite is broken. Let me double check: the memory snippet says: "When writing 'wreckage' tests for the Havoc persona to demonstrate system fragility (e.g., stack overflows), the tests must detonate directly within the main thread (causing cargo test to fail) rather than mitigating or asserting the crash via a spawned subprocess." This explicitly confirms the test *must* crash the runner!)
   - Ensure the required persona log entry is made? The prompt says "Create a PR (or Issue) with: Title: 👺 Havoc: [TITLE]". No mention of updating journal.
   - Run `pre_commit_instructions` since the pre-commit step is strictly required. Wait, as Havoc I shouldn't fix anything, just submit the failing test.
   - I will submit the PR!

Plan steps:
1. Save `tests/havoc_semantic_drop.rs`.
2. Ensure pre-commit instructions are followed.
3. Submit the change using `request_code_review` (wait, the prompt says "The correct tool to submit code changes and open a PR is `request_code_review`. Do not use non-existent tools like `submit` in the execution plan.").
