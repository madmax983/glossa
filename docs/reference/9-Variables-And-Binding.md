# 9. Variables and Binding

## 9.1 Binding with ἔστω

The imperative of εἰμί ("to be") creates bindings. Variable names are Greek letters or words:

```glossa
ξ πέντε ἔστω.
"x five let-it-be"
// → let xi = 5;

ὄνομα «Σωκράτης» ἔστω.
"name 'Socrates' let-it-be"
// → let onoma = "Socrates";

θ [1, 2, 3] ἔστω.
"theta [1, 2, 3] let-it-be"
// → let mut theta = vec![1, 2, 3];
```

Array bindings are automatically made mutable to support push/pop operations.

## 9.2 Greek Letter Variables

Single Greek letters serve as variable names and are transliterated in generated Rust:

| Greek | Rust | Greek | Rust |
|-------|------|-------|------|
| α | alpha | ν | nu |
| β | beta | ξ | xi |
| γ | gamma | ο | omicron |
| δ | delta | π | pi |
| ε | epsilon | ρ | rho |
| ζ | zeta | σ | sigma |
| η | eta | τ | tau |
| θ | theta | υ | upsilon |
| ι | iota | φ | phi |
| κ | kappa | χ | chi |
| λ | lambda | ψ | psi |
| μ | mu | ω | omega |

Multi-letter Greek words are also valid identifiers and are transliterated to Latin characters.

## 9.3 Shadowing

New binding with the same name shadows the previous one:

```glossa
ξ πέντε ἔστω.
ξ «χαῖρε» ἔστω.    // Shadows with new type
```

## 9.4 Mutable Bindings with μετά

By default, bindings are immutable. Use the prefix μετά ("changeable") to create a mutable binding:

```glossa
μετά ξ πέντε ἔστω.
"mutable xi five let-it-be"
// → let mut xi = 5;

μετά ὄνομα «Σωκράτης» ἔστω.
"mutable name 'Socrates' let-it-be"
// → let mut onoma = "Socrates";
```

The word μετά comes from the Greek preposition meaning "after" or "changing," reflecting that the variable's value can change after its initial binding.

Note: Array bindings (`[1, 2, 3] ἔστω`) are automatically mutable to support push/pop operations, even without the μετά prefix.

## 9.5 Assignment with γίγνεται

To reassign a mutable variable, use the middle voice verb γίγνεται ("becomes"):

```glossa
μετά ξ πέντε ἔστω.     // let mut xi = 5;
ξ δέκα γίγνεται.       // xi = 10;
"xi ten becomes"
```

The middle voice γίγνεται (from γίγνομαι, "to become") indicates that the subject undergoes a change—the variable "becomes" its new value. This mirrors how Ancient Greek uses voice to express the relationship between subject and action.

### Error Handling

Attempting to assign to an immutable variable produces an error:

```glossa
ξ πέντε ἔστω.          // let xi = 5; (immutable)
ξ δέκα γίγνεται.       // ERROR: Τὸ «ξ» ἀμετάβλητόν ἐστιν
                       // "xi is unchangeable—use μετά before the definition"
```

Attempting to assign to an undefined variable also produces an error:

```glossa
ξ δέκα γίγνεται.       // ERROR: Τὸ «ξ» οὐχ ὡρίσθη
                       // "xi was not defined—first define it"
```

### Complete Example

```glossa
μετά μετρητής μηδέν ἔστω.    // let mut metritis = 0;
μετρητής λέγε.               // println!("{}", metritis);  → 0
μετρητής ἓν γίγνεται.        // metritis = 1;
μετρητής λέγε.               // println!("{}", metritis);  → 1
```
