# 16. Implementations

## 16.1 Trait Implementation

Trait implementations use **εἶδος** + type name + **τῷ** (dative marker) + trait name + **ἐμπίπτειν** ("to fall into / to implement"):

```glossa
εἶδος Point τῷ Showable ἐμπίπτειν {
    show τῷ self· selfου ξ λέγε.
}.
"form Point for Showable to-implement {
    show for self: self's x say.
}."
```

→

```rust
impl Showable for Point {
    fn show(&self) {
        println!("{}", self.xi);
    }
}
```

Each method starts with its name, parameters (using **τῷ** for each), then a middle dot (`·`) followed by the method body.

## 16.2 Self References

Inside method bodies, **selfου** (genitive of self, "of self") accesses fields:

```glossa
selfου ξ λέγε.
"of-self x say."
// → println!("{}", self.xi);

selfου v otherou v ἄθροισμα
"of-self v of-other v sum"
// → self.v + other.v
```

| Pattern | Meaning | Rust |
|---------|---------|------|
| `τῷ self` | self parameter | `&self` |
| `selfου field` | field of self | `self.field` |
| `τῷ other` | additional parameter | `other: &Self` |
| `otherou field` | field of other | `other.field` |

## 16.3 Complete Example

```glossa
// Define trait
χαρακτήρ Math ὁρίζειν {
    δεῖ add τῷ self τῷ other.
    ἤδη double τῷ self· δός selfου value selfου value ἄθροισμα.
}.

// Define type
εἶδος Number ὁρίζειν { v ἀριθμοῦ. }.

// Implement trait for type
εἶδος Number τῷ Math ἐμπίπτειν {
    add τῷ self τῷ other· δός νέον Number (selfου v otherou v ἄθροισμα).
}.
```

## 16.4 Validation Rules

The compiler enforces:

1. **Trait must be defined** before implementing it
2. **Type must be defined** before implementing a trait for it
3. **All required methods** (δεῖ) must be provided in the implementation
4. **Default methods** (ἤδη) may be omitted (the default body is inherited) or overridden
5. A type may implement **multiple traits**
6. Multiple types may implement the **same trait**