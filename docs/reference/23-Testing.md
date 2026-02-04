# 23. Testing

## 23.1 Test Declarations

GLOSSA provides native test infrastructure using the δοκιμή (dokimē - "test, trial") construct:

```glossa
δοκιμή «test name».
    // test body
τέλος.
```

**Components:**
- **δοκιμή** - Test declaration keyword
- **«test name»** - Test name in Greek guillemets (string literal)
- **τέλος** - End marker

**Transpilation:**
Each δοκιμή block generates a Rust `#[test]` function with a sanitized name:

```glossa
δοκιμή «simple addition».
    // ...
τέλος.
```

Becomes:

```rust
#[test]
fn test_simple_addition() {
    // ...
}
```

## 23.2 Boolean Assertions (δεῖ)

The verb **δεῖ** (dei - "it is necessary") asserts boolean conditions:

```glossa
2 ἐν χ δεῖ.
```

**Morphology:**
- Verb: δέω (deō)
- Form: 3rd person singular, present indicative active
- Meaning: "it is necessary, it must be"
- Voice: Impersonal verb (no explicit subject)

**Semantics:**
Used with containment expressions (`ἐν` - "in"):

```glossa
δοκιμή «HashMap contains key».
    χ νέον χάρτης ἔστω.
    χ 2 0 τίθησι.

    2 ἐν χ δεῖ.        // assert!(chi.contains_key(&2))
τέλος.
```

**Word Order:**
Free word order applies:
```glossa
2 ἐν χ δεῖ.          // element in collection must-be
δεῖ 2 ἐν χ.          // must-be element in collection
ἐν χ 2 δεῖ.          // in collection element must-be
```

All three forms are semantically identical.

## 23.3 Equality Assertions (ἰσοῦται)

The verb **ἰσοῦται** (isoutai - "equals") asserts equality:

```glossa
κ 5 ἰσοῦται.
```

**Morphology:**
- Verb: ἰσόω (isoō - "to make equal")
- Form: 3rd person singular, present indicative middle/passive
- Meaning: "equals, is made equal to"
- Voice: Middle (reflexive action)

**Semantics:**
Compares a variable (nominative) with an expected value:

```glossa
δοκιμή «equality check».
    κ 5 ἔστω.
    κ 5 ἰσοῦται.      // assert_eq!(kappa, 5)
τέλος.
```

**Pattern:**
```
variable value ἰσοῦται
```

- **variable** - Subject (nominative) - the variable being checked
- **value** - Literal value - the expected value
- **ἰσοῦται** - Equality verb

## 23.4 Complete Example

A comprehensive test suite demonstrating both assertion types:

```glossa
δοκιμή «HashMap insert and contains».
    χ νέον χάρτης ἔστω.
    χ 2 0 τίθησι.
    χ 7 1 τίθησι.

    2 ἐν χ δεῖ.
    7 ἐν χ δεῖ.
τέλος.

δοκιμή «HashSet contains elements».
    σ νέον σύνολον ἔστω.
    σ 1 τίθησι.
    σ 2 τίθησι.
    σ 3 τίθησι.

    1 ἐν σ δεῖ.
    2 ἐν σ δεῖ.
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

## 23.5 Running Tests

GLOSSA tests transpile to standard Rust test functions:

```bash
# Compile GLOSSA to Rust
glossa build tests.γλ -o tests.rs

# Run tests
rustc --test tests.rs -o tests
./tests
```

Or using Cargo in a Rust project:

```bash
cargo test
```

## 23.6 Grammar Reference

```pest
test_declaration = {
    ("δοκιμή" | "δοκιμη") ~ string_literal ~ statement_end ~
    test_body ~
    ("τέλος" | "τελος") ~ statement_end
}

test_body = { (!("τέλος" | "τελος") ~ statement)* }
```

**Design Notes:**
- The `test_body` rule uses negative lookahead `!("τέλος" | "τελος")` to prevent the recursive `statement*` from consuming the end marker
- Both accented (δοκιμή, τέλος) and unaccented (δοκιμη, τελος) forms are accepted
- Test names must be string literals in guillemets (`«»`)

## 23.7 Assertion Verb Reference

| Verb | Lemma | Meaning | Rust Equivalent | Use Case |
|------|-------|---------|-----------------|----------|
| δεῖ | δέω | it is necessary | `assert!()` | Boolean conditions, containment |
| ἰσοῦται | ἰσόω | equals | `assert_eq!()` | Value equality |

## 23.8 Limitations

Current implementation supports:
- Boolean assertions (containment checks)
- Equality assertions (value comparison)

**Not Yet Supported:**
- Inequality assertions (`assert_ne!`)
- Ordering comparisons (`assert!(x > y)`)
- Panic expectations (`should_panic`)
- Test fixtures/setup
- Test grouping/modules

See ADR-005 for future considerations.
