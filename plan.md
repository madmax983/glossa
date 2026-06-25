1. Fix the "unused variable: `input`" warning in `src/main.rs` for `Commands::Gnomon { input }` by replacing it with:
```rust
        Some(Commands::Gnomon { input }) => {
            #[cfg(feature = "nova")]
            glossa::tools::gnomon::run_gnomon(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'gnomon' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }
```

2. Make internal modules in `src/semantic/mod.rs` (`pub mod assembly`) restricted to `pub(crate) mod assembly` to enforce better encapsulation per Atlas's persona.

3. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

4. Submit the change with "🗺️ Atlas: Encapsulate Semantic Assembler & Fix Unused Variables"
