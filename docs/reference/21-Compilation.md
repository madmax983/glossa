# 21. Compilation

## 21.1 Pipeline

```
Source (.γλ)
    ↓ PEG Parser
Tokens (flat list)
    ↓ Morphological Analyzer
Analyzed Tokens (with case, tense, mood, etc.)
    ↓ Slot-Based Assembler
Assembled Statements (subject, verb, object, indirect, genitives)
    ↓ Semantic Analyzer
HIR (High-level IR)
    ↓ Rust Codegen
Rust Source (.rs)
    ↓ rustc
Binary
```

## 21.2 File Extension

`.γλ` or `.gl` for source files.

## 21.3 Example Compilation

```glossa
// Source: hello.γλ
«χαῖρε κόσμε» λέγε.
ξ πέντε ἔστω.
ξ λέγε.
```

```rust
// Generated: hello.rs
fn main() {
    println!("{}", "χαῖρε κόσμε");
    let xi = 5;
    println!("{}", xi);
}
```