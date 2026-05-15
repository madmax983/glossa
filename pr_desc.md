🗺️ Atlas: Splitting The Blob: Conversion

🕸️ Tangle: The `src/semantic/conversion.rs` file was a monolithic file (~3000 lines) mixing extraction, classification, orchestration, and tests.
📐 Blueprint: Created `src/semantic/conversion/` module, extracted logic to `classify.rs` and `extract.rs`, moved tests to `tests.rs`.
🧱 Stability: Reduced file size, improved separation of concerns, and maintained strict `pub(crate)` visibility.
🔬 Verification: Builds successfully, all tests pass.
