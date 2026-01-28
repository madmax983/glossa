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
