# Plan

1. **Remove unused `TraitDef` and `TraitImpl` from `src/semantic/model.rs`**:
   - `TraitDef` and `TraitImpl` defined at the bottom of `src/semantic/model.rs` are used only in `Symbol` and `ScopeLevel` inside `src/semantic/resolver.rs` and `Cartographer` logic.
   - However, `AnalyzedStatement::TraitDefinition` already captures the trait data: `{ name: SmolStr, methods: Vec<AnalyzedMethod> }`.
   - Wait, `resolver.rs` defines:
     ```rust
     pub enum Symbol {
         Trait(crate::semantic::model::TraitDef),
     }
     ```
     We can easily inline `TraitDef` into `resolver.rs`, or just replace `TraitDef` with an anonymous struct in the enum variant or a local struct. Wait, wait. Is there any single-implementation trait I can flatten? No. So I will focus on deleting dead code.
     Actually, let's remove `TraitDef` and `TraitImpl` entirely from `model.rs` and create local representations in `resolver.rs` if needed, or simply map them inline. This removes "speculative generality / dead code".

Let's look more closely at `TraitDef`.
Wait, in `src/semantic/model.rs`:
```rust
pub struct TraitDef {
    pub name: SmolStr,
    pub methods: Vec<AnalyzedMethod>,
}

pub struct TraitImpl {
    pub trait_name: SmolStr,
    pub type_name: SmolStr,
}
```
In `src/semantic/resolver.rs`:
```rust
pub enum Symbol {
    ...
    Trait(crate::semantic::model::TraitDef),
}

struct ScopeLevel {
    ...
    trait_impls: Vec<crate::semantic::model::TraitImpl>,
}
```

This is DTO bloat. We can just use named tuple variants or inline them where they are used.

Let's do this:
1. In `src/semantic/resolver.rs`:
   Change `Symbol::Trait(crate::semantic::model::TraitDef)` to:
   ```rust
   pub enum Symbol {
       ...
       Trait {
           name: SmolStr,
           methods: Vec<crate::semantic::model::AnalyzedMethod>,
       },
   }
   ```
   Change `ScopeLevel::trait_impls` to:
   ```rust
   trait_impls: Vec<(SmolStr, SmolStr)>, // trait_name, type_name
   ```
   Fix all the references in `src/semantic/resolver.rs` to match the new definitions.

2. Also fix usages of `TraitDef` and `TraitImpl` in `src/tools/cartographer.rs` and `src/semantic/declarations.rs`.
   - In `declarations.rs`, we currently have `let trait_def_semantic = TraitDef { ... }`. We can just directly instantiate `AnalyzedStatement::TraitDefinition { name, methods }` and pass the data to `Scope`.

3. **Delete `TraitDef` and `TraitImpl` from `src/semantic/model.rs`**.

4. Run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` to ensure it passes.
