# ΓΛΩΣΣΑ (GLOSSA)

[![CI](https://github.com/madmax983/glossa/actions/workflows/ci.yml/badge.svg)](https://github.com/madmax983/glossa/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/madmax983/glossa/branch/trunk/graph/badge.svg)](https://codecov.io/gh/madmax983/glossa)

> *Code as the ancients intended.*

ΓΛΩΣΣΑ is a compiled programming language where Ancient Greek morphology determines semantics. It compiles directly to Rust, offering type safety with authentic linguistic structure.

## The Philosophy

In modern languages, meaning is determined by word order: `func(a, b)` is different from `func(b, a)`.
In Ancient Greek, meaning is determined by **case endings**.

ΓΛΩΣΣΑ embraces this paradigm:

* **Nominative Case** (-ος, -η, -α) marks the **Subject** (Agent).
* **Accusative Case** (-ον, -ην, -αν) marks the **Object** (Patient).
* **Verb Endings** (-ω, -ει, -ετε) encode Person, Number, and Aspect.

This allows for **Free Word Order**:

```glossa
// All of these are identical:
ὁ ἄνθρωπος τὸν λόγον λέγει.  // The man says the word.
τὸν λόγον λέγει ὁ ἄνθρωπος.  // The word says the man.
λέγει ὁ ἄνθρωπος τὸν λόγον.  // Says the man the word.
```

## Quick Start: The Hero's Journey

Here is a simple program that defines a user struct and greets them. Save this code as `hero.γλ`.

```glossa
// Define a type (struct)
εἶδος Χρήστης ὁρίζειν {
    ὄνομα ὀνόματος.      // field: String
    ἡλικία ἀριθμοῦ.   // field: i64
}.

// Create a new user instance
// "user" (nominative) "new" (adjective) "User" (type) ...
χρήστης νέον Χρήστης
    «Σωκράτης»
    70
ἔστω.

// Access property and print
// "of the user" (genitive) "name" (nominative) "say" (verb)
χρήστου ὄνομα λέγε.
```

Run the program:

```bash
cargo run --release -- hero.γλ
```

## Control Flow

### Conditionals

```glossa
εἰ ξ πέντε μείζον,
    «μείζον» λέγε.
```

### Loops

```glossa
// While loop
ἕως ξ δέκα ἔλασσον,
    ξ ξ ἕνα ἄθροισμα γίγνεται. // x = x + 1
```

## Features

- **Greek Syntax**: Write code using authentic Ancient Greek grammatical constructs
- **Type System**: User-defined types (structs) with Greek names
- **Traits**: Interface definitions with default implementations
- **Lambda Expressions**: Participles as closures with multiple capture modes
- **Iterator Operations**: map, filter, find, fold, any, all
- **Control Flow**: Conditionals (εἰ), loops (ἕως), pattern matching
- **Functions**: First-class functions with Greek verb syntax
- **Morphological Analysis**: Full Greek morphology parsing

## Lambda Expressions (Participles)

GLOSSA uses Greek participles as lambda expressions:

```glossa
// Present participle - streaming operation (borrow)
ξ [1, 2, 3] διπλασιαζόμενα λέγε.  // ξ.iter().map(|x| x * 2)

// Aorist participle - one-shot operation (move)
ξ [1, 2, 3] γράψαντα λέγε.        // ξ.into_iter().map(move |x| write(x))

// Perfect participle - memoized operation
ξ [1, 2, 3] κεκαχυμένα λέγε.      // Cached/memoized closure
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

Current test coverage: 294/294 tests passing (100%)

## Documentation

See `docs/reference/` for language reference documentation.

## License

TBD
