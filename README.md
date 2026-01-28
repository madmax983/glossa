# ΓΛΩΣΣΑ (GLOSSA)

[![CI](https://github.com/madmax983/glossa/actions/workflows/ci.yml/badge.svg)](https://github.com/madmax983/glossa/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/madmax983/glossa/branch/trunk/graph/badge.svg)](https://codecov.io/gh/madmax983/glossa)

A programming language using Ancient Greek syntax that compiles to Rust.

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
