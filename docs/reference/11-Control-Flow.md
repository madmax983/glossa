# 11. Control Flow

## 11.1 The Aristotelian Approach

ΓΛΩΣΣΑ follows Aristotle's logical syntax from the *Organon* rather than modern imperative control flow. In Aristotle's syllogistic logic, consequences are introduced with **ἀνάγκη** (necessity):

```
εἰ τὸ Α κατὰ παντὸς τοῦ Β, ἀνάγκη τὸ Α κατὰ παντὸς τοῦ Γ
"If A of all B, necessarily A of all C"
```

This is **declarative**: we state logical truths, not imperative commands.

## 11.2 Conditionals

### Simple Conditional (εἰ... ἀνάγκη)

```glossa
εἰ ξ πέντε μεῖζον ᾖ, ἀνάγκη «μέγα» λέγειν.
"If x than-five greater be-[subj], necessarily 'large' to-say."
// → if x > 5 { println!("large") }

// The subjunctive ᾖ marks the condition boundary
// ἀνάγκη introduces the necessary consequence
```

### Alternative: δεῖ (must/ought)

For less absolute necessity, use **δεῖ** (it is necessary, one must):

```glossa
εἰ ξ πέντε μεῖζον ᾖ, δεῖ «μέγα» λέγειν.
"If x than-five greater be, one-must 'large' to-say."
```

### Conditional with Else (εἰ... ἀνάγκη... εἰ δὲ μή...)

```glossa
εἰ ξ πέντε μεῖζον ᾖ, ἀνάγκη «ναί» λέγειν · εἰ δὲ μή, «οὔ» λέγειν.
"If x than-five greater be, necessarily 'yes' to-say; if but not, 'no' to-say."
// → if x > 5 { println!("yes") } else { println!("no") }
```

### Chained Conditionals

```glossa
εἰ ξ μηδὲν ᾖ, ἀνάγκη «μηδέν» λέγειν ·
εἰ ξ ἓν ᾖ, ἀνάγκη «ἕν» λέγειν ·
εἰ δὲ μή, «ἄλλο» λέγειν.

"If x zero be, necessarily 'zero' to-say;
 if x one be, necessarily 'one' to-say;
 if but not, 'other' to-say."
// → if x == 0 { "zero" } else if x == 1 { "one" } else { "other" }
```

### Conditional Particles

| Greek | Meaning | Usage |
|-------|---------|-------|
| εἰ | if | General conditional |
| ἐάν / ἤν | if (uncertain) | With subjunctive, future-leaning |
| ἀνάγκη | necessarily | Introduces logical consequence |
| δεῖ | must, ought | Introduces practical consequence |
| εἰ δὲ μή | if but not | Else clause |
| ἄλλως | otherwise | Alternative else |

## 11.3 Pattern Matching (κατά)

Pattern matching uses **κατά** (according to) with cases:

```glossa
κατὰ τιμήν·
    μηδὲν ᾖ, ἀνάγκη «μηδέν»·
    ἓν ᾖ, ἀνάγκη «ἕν»·
    ἄλλο ᾖ, ἀνάγκη «ἄλλο».

"According-to value:
    zero be, necessarily 'zero';
    one be, necessarily 'one';
    other be, necessarily 'other'."

// → match value { 0 => "zero", 1 => "one", _ => "other" }
```

## 11.4 Loops

### While Loop (ἕως... ἀνάγκη)

The pattern follows Aristotle: "while condition holds, necessarily action":

```glossa
ἕως ξ μηδενὸς μεῖζον ᾖ, ἀνάγκη ξ μειοῦσθαι.
"While x than-zero greater be-[subj], necessarily x to-decrease."
// → while x > 0 { x -= 1 }
```

### Iteration (διά + Genitive)

Iteration uses **διά** (through) with the genitive:

```glossa
διὰ στοιχείων λίστης, ἀνάγκη στοιχεῖον λέγειν.
"Through elements of-list, necessarily element to-say."
// → for elem in list { println!("{}", elem) }
```

### Range Iteration (ἀπό... μέχρι/ἕως)

```glossa
ἀπὸ μηδενὸς μέχρι δέκα, ἀνάγκη ι λέγειν.
"From zero until ten, necessarily i to-say."
// → for i in 0..10 { println!("{}", i) }

ἀπὸ μηδενὸς ἕως δέκα, ἀνάγκη ι λέγειν.
"From zero to ten [inclusive], necessarily i to-say."
// → for i in 0..=10 { println!("{}", i) }
```

## 11.5 Loop Control

```glossa
παῦε.           // break - "stop!" (imperative of παύω)
συνέχιζε.       // continue - "carry on!" (imperative of συνεχίζω)
```

## 11.6 Early Return

```glossa
δίδου τιμήν.    // return value - "give value!" (imperative active)
διδόσθω τιμή.   // return value - "let value be given" (imperative passive)
```

## 11.7 Design Rationale

### Why ἀνάγκη?

1. **Philosophical consistency**: Follows Aristotle's actual logical notation
2. **Declarative semantics**: States what must be true, not what to do
3. **Clear boundaries**: ἀνάγκη explicitly marks the consequence
4. **No braces needed**: Greek syntax naturally delimits clauses

### Comparison with Traditional Languages

| Traditional | ΓΛΩΣΣΑ |
|-------------|--------|
| `if (x > 5) { ... }` | `εἰ ξ πέντε μεῖζον ᾖ, ἀνάγκη ...` |
| `while (x > 0) { ... }` | `ἕως ξ μηδενὸς μεῖζον ᾖ, ἀνάγκη ...` |
| `for x in list { ... }` | `διὰ στοιχείων, ἀνάγκη ...` |

The subjunctive mood (ᾖ) naturally marks hypothetical/conditional boundaries,
and ἀνάγκη introduces what necessarily follows—exactly as Aristotle wrote.
