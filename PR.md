Title: 🗺️ Atlas: [Enforce Module Encapsulation for Internal Tools and Assembler]
Description:
🕸️ Tangle: Several internal tools (cartographer, catalog, dictionary, haruspex) and the semantic assembly pipeline were fully public, leaking internal implementation details.
📐 Blueprint: Changed `pub mod` to `pub(crate) mod` for these modules to restrict them to crate visibility.
🧱 Stability: Reduced public API surface, enforcing clear boundaries.
🔬 Verification: Builds successfully, all tests pass, and strict separation is enforced.
