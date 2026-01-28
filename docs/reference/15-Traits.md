# 15. Traits (Χαρακτήρ)

## 15.1 Trait Definition

Traits are defined with **χαρακτήρ** ("characteristic/trait") + name + **ὁρίζειν** + braced methods:

```glossa
χαρακτήρ Showable ὁρίζειν {
    δεῖ show τῷ self.
}.
"characteristic Showable to-define {
    must show for self.
}."
```

→

```rust
trait Showable {
    fn show(&self);
}
```

## 15.2 Required Methods (δεῖ)

**δεῖ** ("it is necessary / must") marks a method that implementors are required to provide:

```glossa
δεῖ show τῷ self.
// → fn show(&self);

δεῖ add τῷ self τῷ other.
// → fn add(&self, other: &Self);
```

Parameters use the **dative marker** τῷ. The first parameter is typically `self`.

## 15.3 Default Methods (ἤδη)

**ἤδη** ("already") marks a method with a default body. The body follows a middle dot (`·`):

```glossa
ἤδη double τῷ self· δός selfου value selfου value ἄθροισμα.
"already double for self: give self's-value self's-value sum."
// → fn double(&self) -> i64 { return self.value() + self.value(); }
```

Default methods need not be implemented by types — they inherit the default body.

## 15.4 Multiple Methods

```glossa
χαρακτήρ Math ὁρίζειν {
    δεῖ add τῷ self τῷ other.
    ἤδη double τῷ self· δός selfου value selfου value ἄθροισμα.
}.
```

→

```rust
trait Math {
    fn add(&self, other: &Self);
    fn double(&self) -> i64 { return self.value() + self.value(); }
}
```