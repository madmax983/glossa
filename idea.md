## 🌟 The Index (ὁ Κατάλογος / The Scribe / API Doc Gen)

**The Spark:** We have a semantic assembly engine that can translate Ancient Greek logic into programmatic types (`GlossaType`), and we collect all definitions in the `Scope`. But it's hard to read a big Glossa file and quickly see all the structs (`εἶδος`) and functions (`ἔργον`) it exports.

**The Feature:** Implement `glossa scribe` (or "Index"). It reads a Glossa file, parses it into an `AnalyzedProgram`, iterates over the `Scope`'s exported `types()`, `functions()`, and `traits()`, and generates a beautiful, standardized Markdown documentation file (like `rustdoc`).

**The Potential:** Users can run `cargo run --features nova -- scribe my_lib.γλ` and get `my_lib.md` out, detailing exactly what types exist, what fields they have, and what functions are available with their signatures.

**Risk:** Low. Isolated in `src/tools/scribe.rs` (behind `nova`), registered in `cli.rs` and `runner.rs`. Just loops over `program.scope`.

Wait, looking at `AnalyzedProgram`, it's not the `scope` we want to iterate entirely (since `scope` is flat and scoped), but actually `program.statements`. But wait, `AnalyzedProgram` has `scope`, which tracks all defined types, traits, and functions globally for that file. Yes!

Let's do this! A tool called "Scribe" (ὁ Γραμματεύς) that generates API documentation in Markdown.

I will create a tool `scribe` that outputs Markdown.
