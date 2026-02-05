# 5. Test Infrastructure with Assertion Verbs

Date: 2025-02-04
Status: Accepted

## Context

Most programming languages provide separate testing frameworks (unittest, pytest, JUnit) with assertion functions (`assert()`, `assertEquals()`). These typically use positional syntax and imperative function calls.

For GLOSSA to be a complete, practical programming language, we need a way to write unit tests that:
1. Transpiles to Rust's native `#[test]` functions
2. Uses idiomatic Ancient Greek syntax consistent with the language's morphological paradigm
3. Leverages existing Greek verbs with appropriate semantic meanings
4. Maintains word-order independence where appropriate

## Decision

We have implemented a native test infrastructure using Greek verb forms:

### Test Declaration Blocks

```glossa
δοκιμή «test name».
    // test body statements
τέλος.
```

- **δοκιμή** (dokimē) - "test, trial, proof" - marks the beginning of a test block
- **τέλος** (telos) - "end, completion" - marks the end of a test block
- Test names are provided in guillemets (`«»`) as string literals

These transpile to Rust `#[test]` functions with sanitized names.

### Assertion Verbs

**Boolean Assertions (δεῖ):**
```glossa
2 ἐν χ δεῖ.  // assert!(chi.contains_key(&2))
```

- **δεῖ** (dei) - 3rd person singular of δέω, "it is necessary, it must be"
- Impersonal verb form appropriate for logical necessity
- Generates `assert!()` macro in Rust

**Equality Assertions (ἰσοῦται):**
```glossa
κ 5 ἰσοῦται.  // assert_eq!(kappa, 5)
```

- **ἰσοῦται** (isoutai) - 3rd person middle/passive of ἰσόω, "equals, is made equal to"
- Middle voice reflects the reflexive nature of equality
- Generates `assert_eq!()` macro in Rust

### Grammar Implementation

The grammar uses negative lookahead to prevent τέλος consumption:

```pest
test_declaration = {
    ("δοκιμή" | "δοκιμη") ~ string_literal ~ statement_end ~
    test_body ~
    ("τέλος" | "τελος") ~ statement_end
}

test_body = { (!("τέλος" | "τελος") ~ statement)* }
```

This ensures the recursive `statement*` rule doesn't consume the ending marker.

### Semantic Analysis

Two new classification functions handle assertions:
- `classify_assertion()` - recognizes δεῖ verb and generates `Assert` IR node
- `classify_equality_assertion()` - recognizes ἰσοῦται verb and generates `AssertEq` IR node

Both integrate into the existing slot-based semantic assembly pipeline.

## Consequences

### Positive

- **Idiomatic Greek**: Uses authentic Greek verbs with appropriate meanings for their functions
- **Type Safety**: Transpiles to Rust's type-safe test infrastructure
- **Consistency**: Follows GLOSSA's morphological paradigm (δεῖ is an impersonal verb, appropriate for logical necessity)
- **Tooling**: Generated tests work with standard Rust tools (`cargo test`, `rustc --test`)
- **Completeness**: Makes GLOSSA suitable for TDD workflows

### Negative

- **Limited Assertion Vocabulary**: Currently only boolean and equality assertions (no `assert_ne!`, `assert!(a > b)`, etc.)
- **Parser Complexity**: Negative lookahead adds complexity to prevent τέλος consumption
- **Two Assertion Patterns**: Users must learn which verb form to use for which assertion type

### Future Considerations

- Add more assertion verbs (inequality, containment, panic expectations)
- Support for test fixtures/setup using participle forms
- Integration with property-based testing (ὑπόθεσις?)
- Test organization/grouping (modules?)

## Examples

Full working example from `examples/working_tests.γλ`:

```glossa
δοκιμή «HashMap insert and contains».
    χ νέον χάρτης ἔστω.
    χ 2 0 τίθησι.

    2 ἐν χ δεῖ.
τέλος.

δοκιμή «mutable counter increments».
    μετά κ 0 ἔστω.
    κ 0 ἰσοῦται.

    κ 1 γίγνεται.
    κ 1 ἰσοῦται.

    κ 5 γίγνεται.
    κ 5 ἰσοῦται.
τέλος.
```

Generates:

```rust
#[test]
fn test_hashmap_insert_and_contains() {
    let mut chi = HashMap::new();
    chi.insert(2i64, 0i64);
    assert!(chi.contains_key(&2i64));
}

#[test]
fn test_mutable_counter_increments() {
    let mut kappa = 0i64;
    assert_eq!(kappa, 0i64);
    kappa = 1i64;
    assert_eq!(kappa, 1i64);
    kappa = 5i64;
    assert_eq!(kappa, 5i64);
}
```
