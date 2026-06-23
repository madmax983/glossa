# ΓΛΩΣΣΑ (GLOSSA)

[![CI](https://github.com/madmax983/glossa/actions/workflows/ci.yml/badge.svg)](https://github.com/madmax983/glossa/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/madmax983/glossa/branch/trunk/graph/badge.svg)](https://codecov.io/gh/madmax983/glossa)

> *Code as the ancients intended.*

ΓΛΩΣΣΑ is a compiled programming language where Ancient Greek morphology determines semantics. It compiles directly to Rust, offering type safety with authentic linguistic structure.

## The Philosophy

In modern languages, meaning is enslaved by position. `func(a, b)` means something entirely different from `func(b, a)`, purely because of where the words sit. This is a tyranny of order.

In Ancient Greek, meaning is liberated by **morphology**. A word's ending tells you its role.
* **-ος** (Nominative) says "I am the subject."
* **-ον** (Accusative) says "I am the object."
* **-ει** (Verb) says "He/She/It does."

ΓΛΩΣΣΑ embraces this freedom. The following lines of code are identical to the compiler:

```glossa
ὁ ἄνθρωπος τὸν λόγον λέγει.  // The man says the word.
τὸν λόγον λέγει ὁ ἄνθρωπος.  // The word says the man.
λέγει ὁ ἄνθρωπος τὸν λόγον.  // Says the man the word.
```

## Quick Start: The Hero's Journey

> 💡 **Tip:** You can find the complete code for this journey in `examples/quickstart.γλ`.

### Chapter 1: The Definition (Variables)

In ΓΛΩΣΣΑ, we define reality with `ἔστω` ("let there be").

```glossa
// Let x be 10.
ξ 10 ἔστω.

// Let name be "Socrates".
ὄνομα «Σωκράτης» ἔστω.
```

### Chapter 2: The Action (Functions)

Verbs drive the action. We use the imperative mood for commands.

```glossa
// Say "Hello"
«χαῖρε» λέγε.

// Say the name (using the variable from Chapter 1)
ὄνομα λέγε.
```

### Chapter 3: The Structure (Types)

We define the shape of our world with `εἶδος` (form/type).

```glossa
// Define a User type
εἶδος Χρήστης ὁρίζειν {
    ὄνομα ὀνόματος.    // String
    ἡλικία ἀριθμοῦ. // i64
}.

// Create an instance
χρήστης νέον Χρήστης
    «Πλάτων»
    80
ἔστω.
```

### Chapter 4: The Logic (Control Flow)

We guide the flow of fate with `εἰ` (if) and `διὰ` (through/for).

```glossa
// Let age be 80
ἡλικία 80 ἔστω.

// If age is greater than 50...
εἰ ἡλικία 50 μεῖζον ᾖ,
    «σοφός» λέγε.
```

## Rosetta Stone

A guide for travelers from other lands.

| Concept | Rust / Python | ΓΛΩΣΣΑ | Literal Meaning |
|---------|---------------|--------|-----------------|
| **Variable** | `let x = 5;` | `ξ 5 ἔστω.` | "Let x be 5." |
| **Print** | `println!("Hi");` | `«χαῖρε» λέγε.` | "Say 'Hi'." |
| **If** | `if x > 0 { ... }` | `εἰ ξ 0 μεῖζον ᾖ, ...` | "If x [is] greater than 0..." |
| **Loop** | `for n in numbers { ... }` | `ἀριθμός [1, 2, 3] ἔστω. διὰ ἀριθμοῦ, ν λέγε.` | "Through numbers, say n." |
| **Function** | `fn foo() { ... }` | `... ὁρίζειν ...` | "To define..." |
| **Struct** | `struct User { ... }` | `εἶδος Χρήστης ...` | "Form User..." |

## Troubleshooting

The compiler speaks to you in Greek. Do not fear it; learn from it.

| Error Message | Translation | What it means | How to fix |
|---------------|-------------|---------------|------------|
| **Ἀσυμφωνία** | Disagreement | Subject/Verb mismatch | Check if your Noun is Singular but Verb is Plural. |
| **Διπλοῦν ὑποκείμενον** | Double Subject | Two Nominatives | You have two subjects (e.g., "The man the god says"). Remove one. |
| **Ῥῆμα οὐχ εὑρέθη** | Verb not found | Missing verb | Every sentence needs a verb (action). Add one. |

## Running Code

To run a ΓΛΩΣΣΑ file (e.g., `hero.γλ`), use `cargo run`:

```bash
cargo run --release -- hero.γλ
```

To run the Quick Start example:

```bash
cargo run --release -- examples/quickstart.γλ
```

## The Nova Toolset (Experimental)

Unlock advanced developer tools by enabling the `nova` feature.

> ⚠️ **REQUIRES FEATURE NOVA**
>
> To use these tools, you must add `--features nova` to your command.

### 1. The Mentor (Interactive Tutorial)
Learn ΓΛΩΣΣΑ step-by-step.

```bash
cargo run --release --features nova -- mentor
```

### 2. The Cartographer (Architecture Map)
Generate a Mermaid.js class diagram of your code.

```bash
cargo run --release --features nova -- map examples/quickstart.γλ
```

### 3. The Mosaic (Sentence Structure)
Visualize how the compiler assembles your sentences.

```bash
cargo run --release --features nova -- mosaic examples/quickstart.γλ
```

### 4. The Bard (Scroll of Logic)
Translate your code into an English narrative (available without `nova`).

```bash
cargo run --release -- bard examples/quickstart.γλ
```

## Features

- **Greek Syntax**: Write code using authentic Ancient Greek grammatical constructs
- **Type System**: User-defined types (structs) with Greek names
- **Traits**: Interface definitions with default implementations
- **Lambda Expressions**: Participles as closures with multiple capture modes
- **Iterator Operations**: map, filter, find, fold, any, all
- **Control Flow**: Conditionals (εἰ), loops (ἕως), pattern matching
- **Functions**: First-class functions with Greek verb syntax
- **Testing Framework**: Native test declarations with assertion verbs (δοκιμή, δεῖ, ἰσοῦται)
- **Morphological Analysis**: Full Greek morphology parsing

## Testing

GLOSSA provides native test declarations using idiomatic Greek verbs:

```glossa
δοκιμή «HashMap insert and contains».
    χ νέον χάρτης ἔστω.
    χ 2 0 τίθησι.

    2 ἐν χ δεῖ.        // assert!(chi.contains_key(&2))
τέλος.

δοκιμή «equality check».
    κ 5 ἔστω.
    κ 5 ἰσοῦται.      // assert_eq!(kappa, 5)
τέλος.
```

**Assertion Verbs:**
- **δεῖ** - "it is necessary" → `assert!(condition)`
- **ἰσοῦται** - "equals" → `assert_eq!(left, right)`

Tests transpile to Rust `#[test]` functions and can be run with standard Rust tools.

### Running Glossa Tests

To run tests written in Glossa (e.g., `my_tests.γλ`):

```bash
cargo run --release -- test my_tests.γλ
```

## Compiler Development

### Building the Compiler

```bash
cargo build --release
```

### Running Compiler Tests

```bash
cargo test
```

Current test coverage: 294/294 tests passing (100%)

## Documentation

See `docs/reference/` for language reference documentation.

## License

TBD
