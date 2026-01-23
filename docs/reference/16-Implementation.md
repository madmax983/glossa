# 16. Implementations

## 16.1 Trait Implementation

Dative (for type) + nominative (trait):

```glossa
Χρήστῃ Ἐμφανίσιμον·
    ἐμφάνισις·
        τούτου ὄνομα δίδωσι.

"For-User Displayable:
    display:
        of-this name gives."
```

→

```rust
impl Emphanisimon for Chrestes {
    fn emphanisis(&self) -> String {
        self.onoma.clone()
    }
}
```

## 16.2 Inherent Implementation

Dative + ἔργα (works):

```glossa
Χρήστῃ ἔργα·
    
    νέος ἐξ ὀνόματος καὶ ἡλικίας ποιεῖται·
        Χρήστης· ὄνομα τὸ ὄνομα· ἡλικία ἡ ἡλικία.
    
    γενέθλια τούτῳ·
        τούτου ἡλικία αὐξάνεται.

"For-User works:
    
    new from name and age makes-itself:
        User: name the name; age the age.
    
    birthday for-this:
        of-this age increases."
```

→

```rust
impl Chrestes {
    fn neos(onoma: String, helikia: i64) -> Self {
        Chrestes { onoma, helikia }
    }
    
    fn genethlia(&mut self) {
        self.helikia += 1;
    }
}
```

## 16.3 Self References

| Greek | Case | Meaning | Rust |
|-------|------|---------|------|
| τοῦτο | Nom | this (subject) | `self` |
| τούτου | Gen | of this | `self.` |
| τούτῳ | Dat | for/to this | `&mut self` |
| τοῦτον | Acc | this (object) | `self` |
| ἑαυτόν | Acc | oneself (reflexive) | `self` |