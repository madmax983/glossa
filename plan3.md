Ah, the `_ => "Unknown"` arm in `mosaic.rs` is actually unreachable in practice because `Statement` only has:
`Regular`, `TypeDefinition`, `TraitDefinition`, `TraitImpl`, `TestDeclaration`.
And `if let Regular` handles `Regular`, leaving only the other 4 which are explicitly matched.
So `_ => "Unknown"` is genuinely unreachable dead code.

But how do we fix the coverage?
Let's see: `src/tools/mosaic.rs` line 120.
```rust
                crate::ast::Statement::TraitImpl(_) => "Trait Implementation",
                crate::ast::Statement::TestDeclaration(_) => "Test Declaration",
                _ => "Unknown", // This is line 120
```

Wait, we should just delete `_ => "Unknown",` and the fallback is exhaustive if we just use `unreachable!()` or remove `_` entirely if Rust figures out it's exhaustive!
BUT wait, `Regular` is matched by `if let` first!
If we match `stmt` inside the `else` block, `Regular` is already handled, but `match stmt` requires `Regular` or `_` if `Regular` isn't listed.
So `_ => "Unknown"` is actually for `Regular`? No, `Regular` cannot enter `else`!
Since `stmt` is not `Regular`, the `match stmt` must cover the other 4. But the Rust compiler requires all variants of `Statement` to be handled in `match stmt` because it doesn't know `stmt` can't be `Regular` here.
So we can replace `_ => "Unknown"` with `_ => unreachable!()`.
Wait, `unreachable!()` might panic and reduce coverage if not hit.
Instead, we can just say `_ => ""` and ignore it. But `_ => ""` still requires hitting to get 100% coverage.
Since `unreachable!()` generates panic code which drops coverage, what if we use:
`crate::ast::Statement::Regular { .. } => unreachable!(),`? Same problem.

What if we refactor `if let` into `match`?
```rust
    for (i, stmt) in program.statements.iter().enumerate() {
        match stmt {
            crate::ast::Statement::Regular { .. } => {
                match assemble_statement(stmt) { ... }
            }
            crate::ast::Statement::TypeDefinition(_) => {
                table.add_row(vec![..., Cell::new("Type Definition")...]);
            }
            crate::ast::Statement::TraitDefinition(_) => ...
            crate::ast::Statement::TraitImpl(_) => ...
            crate::ast::Statement::TestDeclaration(_) => ...
        }
    }
```
If we do this, the match is exhaustive without an `_` catch-all.
Let's refactor lines 94-123 in `src/tools/mosaic.rs` to an exhaustive `match stmt`.
