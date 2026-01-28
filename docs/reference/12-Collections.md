# 12. Collections

## 12.1 Arrays

### Literals

```glossa
ξ [1, 2, 3] ἔστω.
"x [1, 2, 3] let-be."
// → let mut xi = vec![1, 2, 3];
```

Array bindings are automatically mutable in generated Rust.

### Indexing with Ordinals

Ordinal adjectives map to array indices:

```glossa
ξ [10, 20, 30] ἔστω.
ψ ξ πρῶτον ἔστω.           // y = x[0]  (first → index 0)
χ ξ δεύτερον ἔστω.          // z = x[1]  (second → index 1)
ξ τρίτον λέγε.              // print(x[2]) (third → index 2)
```

| Ordinal | Meaning | Index |
|---------|---------|-------|
| πρῶτον | first | 0 |
| δεύτερον | second | 1 |
| τρίτον | third | 2 |

### Direct Indexing

Square bracket indexing is also supported:

```glossa
ξ[0] λέγε.
// → println!("{}", xi[0]);
```

### Length

```glossa
ξ μῆκος λέγε.
"x length say."
// → println!("{}", xi.len());
```

## 12.2 Iterator Operations

See [Chapter 17 — Participles as Lambdas](17-Lambdas.md) for map, filter, fold, find, any, and all operations on collections.

```glossa
[1, 2, 3] διπλασιαζόμενα λέγε.         // map: double each
[1, 10, 3, 8] πέντε μείζονα λέγε.       // filter: > 5
[1, 5, 3, 8] πέντε μείζονα διπλασιαζόμενα λέγε.  // filter then map
```

> **Not yet implemented:** HashMap, Set, and String-specific operations (insert, get, contains, split, join). The lexicon recognizes the Greek vocabulary for these types (χάρτης, σύνολον, λόγος) but semantic support for their operations is pending.