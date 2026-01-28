# 2. Lexical Structure

## 2.1 Character Set

ΓΛΩΣΣΑ source files are UTF-8 encoded. The primary alphabet is Greek (including polytonic diacritics), with ASCII transliteration supported.

### Greek Alphabet

| Letter | Name | ASCII | As Numeral |
|--------|------|-------|------------|
| Α α | alpha | a | 1 |
| Β β | beta | b | 2 |
| Γ γ | gamma | g | 3 |
| Δ δ | delta | d | 4 |
| Ε ε | epsilon | e | 5 |
| Ζ ζ | zeta | z | 7 |
| Η η | eta | ē, ee | 8 |
| Θ θ | theta | th | 9 |
| Ι ι | iota | i | 10 |
| Κ κ | kappa | k | 20 |
| Λ λ | lambda | l | 30 |
| Μ μ | mu | m | 40 |
| Ν ν | nu | n | 50 |
| Ξ ξ | xi | x | 60 |
| Ο ο | omicron | o | 70 |
| Π π | pi | p | 80 |
| Ρ ρ | rho | r | 100 |
| Σ σ/ς | sigma | s | 200 |
| Τ τ | tau | t | 300 |
| Υ υ | upsilon | u, y | 400 |
| Φ φ | phi | ph | 500 |
| Χ χ | chi | ch, kh | 600 |
| Ψ ψ | psi | ps | 700 |
| Ω ω | omega | ō, oo | 800 |

### Diacritics

Diacritics (breathings, accents, iota subscript) are normalized during lexing but preserved for display:

- Smooth breathing (ἀ) → α
- Rough breathing (ἁ) → α  
- Acute accent (ά) → α
- Grave accent (ὰ) → α
- Circumflex (ᾶ) → α
- Iota subscript (ᾳ) → α

## 2.2 Punctuation

ΓΛΩΣΣΑ uses Greek punctuation:

| Symbol | Unicode | Name | Meaning |
|--------|---------|------|---------|
| `.` | U+002E | τελεία | Statement terminator |
| `·` | U+00B7 | ἄνω τελεία | Expression chain (chains clauses) |
| `;` | U+037E | ἐρωτηματικό | Query (also accepts `?`) |
| `;` | U+003B | (ASCII semicolon) | Propagation operator (Rust's `?`) |
| `,` | U+002C | (comma) | Clause separator (control flow) |
| `«»` | U+00AB/BB | εἰσαγωγικά | String delimiters |

## 2.3 Literals

### Strings
```glossa
«χαῖρε κόσμε»           // Greek guillemets
```

String literals use Greek guillemets (`« »`). ASCII double quotes are not supported.

### Numbers
```glossa
42                      // Arabic numerals (integer only)
```

> **Not yet implemented:** Hexadecimal (`0x2A`), binary (`0b101010`), floating-point (`3.14`), and Greek numeral (`μβʹ`) literals.

### Booleans
```glossa
ἀληθές / αληθες         // true
ψεῦδος / ψευδος         // false
```

### Null/None
```glossa
οὐδέν / ουδεν           // nothing, none, null
```

## 2.4 Comments

```glossa
// Single line comment (like Rust)
```

> **Not yet implemented:** Block comments (`/* ... */`).

## 2.5 Whitespace

Whitespace separates tokens but has no semantic meaning. Line breaks do not terminate statements (only `.` does).