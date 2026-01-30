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

## 12.3 HashSet Operations

HashSet (σύνολον - "set/collection") provides unique element storage.

### Creating a HashSet

```glossa
ξ νέον σύνολον ἔστω.
"x new set let-be."
// → let mut xi = HashSet::new();
```

### Insert

```glossa
ξ 42 τίθησι.
"x 42 places."
// → xi.insert(42);
```

The verb τίθημι (τίθησι = "places") maps to `.insert()`.

### Contains

```glossa
42 ἐν ξ?
"42 in x?"
// → xi.contains(&42)
```

The preposition ἐν ("in") with a query (`?`) generates a `.contains()` check.

## 12.4 HashMap Operations

HashMap (χάρτης - "map/chart") provides key-value storage.

### Creating a HashMap

```glossa
ξ νέον χάρτης ἔστω.
"x new map let-be."
// → let mut xi = HashMap::new();
```

### Insert

```glossa
ξ «ὄνομα» «Σωκράτης» τίθησι.
"x name Socrates places."
// → xi.insert("ὄνομα", "Σωκράτης");
```

With two arguments, τίθησι generates `.insert(key, value)`.

### Contains Key

```glossa
«ὄνομα» ἐν ξ?
"name in x?"
// → xi.contains_key(&"ὄνομα")
```

The ἐν pattern with HashMap generates `.contains_key()` instead of `.contains()`.

## 12.5 String Operations

### Contains

```glossa
ξ «χαῖρε κόσμε» ἔστω.
«κόσμε» ἐν ξ?
"kosme in x?"
// → xi.contains("κόσμε")
```

The ἐν pattern works with strings to check for substrings.

### Split

```glossa
ξ «χαῖρε-κόσμε» ἔστω.
ξ κατὰ «-» σχίζεται λέγε.
"x by dash splits-itself say."
// → println!("{}", xi.split("-"));
```

The preposition κατά ("by/according to") with the middle voice verb σχίζεται ("splits-itself") generates `.split(delimiter)`.

### Join

```glossa
ξ [«α», «β», «γ»] ἔστω.
ξ κατὰ «-» ἑνοῦνται λέγε.
"x by dash unite-themselves say."
// → println!("{}", xi.join("-"));
```

The verb ἑνοῦνται ("they unite") generates `.join(delimiter)`.

## 12.6 Vocabulary Summary

| Greek | Meaning | Rust Equivalent |
|-------|---------|-----------------|
| σύνολον | set | `HashSet` |
| χάρτης | map | `HashMap` |
| τίθησι | places | `.insert()` |
| ἐν ... ? | in ...? | `.contains()` / `.contains_key()` |
| κατὰ ... σχίζεται | by ... splits | `.split()` |
| κατὰ ... ἑνοῦνται | by ... unite | `.join()` |