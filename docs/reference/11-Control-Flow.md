# 11. Control Flow

## 11.1 Design Philosophy

ΓΛΩΣΣΑ uses Greek grammatical mood to delimit control flow:

- The **subjunctive mood** (ᾖ) marks hypothetical/conditional boundaries
- **Commas** separate condition clauses from consequence clauses
- **Imperative verbs** (λέγε, παῦε) express commands in consequence clauses
- **εἰ δὲ μή** ("if but not") introduces else branches

This allows natural Greek sentence structure to encode if/else, loops, and pattern matching without braces or special delimiters.

## 11.2 Conditionals

### Simple Conditional (εἰ... ᾖ)

```glossa
εἰ ξ πέντε μεῖζον ᾖ, «ναί» λέγε.
"If x than-five greater be-[subj], 'yes' say."
// → if x > 5 { println!("yes") }

// The subjunctive ᾖ marks the condition boundary
// The comma separates condition from consequence
```

### Conditional with Else (εἰ... εἰ δὲ μή...)

```glossa
εἰ ξ πέντε μεῖζον ᾖ, «ναί» λέγε · εἰ δὲ μή, «οὔ» λέγε.
"If x than-five greater be, 'yes' say; if but not, 'no' say."
// → if x > 5 { println!("yes") } else { println!("no") }
```

### Chained Conditionals

```glossa
εἰ ξ μηδὲν ᾖ, «μηδέν» λέγε ·
εἰ ξ ἓν ᾖ, «ἕν» λέγε ·
εἰ δὲ μή, «ἄλλο» λέγε.

"If x zero be, 'zero' say;
 if x one be, 'one' say;
 if but not, 'other' say."
// → if x == 0 { "zero" } else if x == 1 { "one" } else { "other" }
```

### Conditional Particles

| Greek | Meaning | Usage |
|-------|---------|-------|
| εἰ | if | General conditional |
| ἐάν / ἤν | if (uncertain) | With subjunctive, future-leaning |
| εἰ δὲ μή | if but not | Else clause |

## 11.3 Pattern Matching (κατά)

Pattern matching uses **κατά** (according to) with cases:

```glossa
κατὰ ξ·
    μηδὲν ᾖ, «μηδέν» λέγε·
    ἓν ᾖ, «ἕν» λέγε·
    ἄλλο ᾖ, «ἄλλο» λέγε.

"According-to x:
    zero be, 'zero' say;
    one be, 'one' say;
    other be, 'other' say."

// → match x { 0 => println!("zero"), 1 => println!("one"), _ => println!("other") }
```

The wildcard pattern uses **ἄλλο** ("other") which maps to Rust's `_`.

## 11.4 Loops

### While Loop (ἕως)

```glossa
ἕως ξ μηδενὸς μεῖζον ᾖ, ξ λέγε.
"While x than-zero greater be-[subj], x say."
// → while x > 0 { println!("{}", x) }
```

### Collection Iteration (διά)

Iteration uses **διά** (through):

```glossa
διὰ στοιχείων, στοιχεῖον λέγε.
"Through elements, element say."
// → for elem in elements { println!("{}", elem) }
```

### Range Iteration (ἀπό... μέχρι/ἕως)

```glossa
ἀπὸ μηδενὸς μέχρι πέντε, ι λέγε.
"From zero until five, i say."
// → for i in 0..5 { println!("{}", i) }

ἀπὸ μηδενὸς ἕως πέντε, ι λέγε.
"From zero to five [inclusive], i say."
// → for i in 0..=5 { println!("{}", i) }
```

Note: **μέχρι** (until) produces an exclusive range (`..`), while **ἕως** (to) produces an inclusive range (`..=`).

## 11.5 Loop Control

```glossa
παῦε.           // break - "stop!" (imperative of παύω)
συνέχιζε.       // continue - "carry on!" (imperative of συνεχίζω)
```

These can be used inside conditional clauses within loops:

```glossa
ἀπὸ μηδενὸς μέχρι δέκα, εἰ ι πέντε μεῖζον ᾖ, παῦε.
// → for i in 0..10 { if i > 5 { break } }
```

## 11.6 Early Return

```glossa
δός τιμήν.      // return value - "give!" (aorist imperative of δίδωμι)
// → return value;
```

## 11.7 Design Rationale

### Why Subjunctive + Comma?

1. **Grammatical naturalism**: The subjunctive mood (ᾖ) naturally marks hypothetical conditions in Greek
2. **Minimal syntax**: Commas separate clauses just as in Greek prose
3. **No braces needed**: Greek clause structure naturally delimits scope
4. **Morphological consistency**: Mood encodes semantics, not keywords

### Comparison with Traditional Languages

| Traditional | ΓΛΩΣΣΑ |
|-------------|--------|
| `if (x > 5) { ... }` | `εἰ ξ πέντε μεῖζον ᾖ, ...` |
| `while (x > 0) { ... }` | `ἕως ξ μηδενὸς μεῖζον ᾖ, ...` |
| `for i in 0..10 { ... }` | `ἀπὸ μηδενὸς μέχρι δέκα, ...` |
| `for x in list { ... }` | `διὰ στοιχείων, ...` |
| `match x { ... }` | `κατὰ ξ· ...` |
