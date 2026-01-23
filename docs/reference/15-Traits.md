# 15. Traits (Ἀρετή)

## 15.1 Trait Definition

Nominative + ἀρετή pattern:

```glossa
Ἐμφανίσιμον ἀρετή·
    ἐμφάνισις λόγον δίδωσι.

"Displayable [is] a virtue:
    display gives string."
```

→

```rust
trait Emphanisimon {
    fn emphanisis(&self) -> String;
}
```

## 15.2 Method Signatures

```glossa
// fn method(&self) -> T
μέθοδος τύπον δίδωσι.

// fn method(&self, param: P) -> T
μέθοδος παραμέτρῳ τύπον δίδωσι.

// fn method(&mut self)
μέθοδος ἑαυτὸν μεταβάλλει.

// fn method(&mut self, param: P)
μέθοδος παραμέτρῳ ἑαυτὸν μεταβάλλει.
```

## 15.3 Trait Bounds

```glossa
// Generic with trait bound
μέγιστον ἔργον· αῳ βῳ Συγκρισίμοις·
    ...

"maximum [is] a work: for-a for-b [which are] Comparable:
    ..."

// → fn max<T: Ord>(a: T, b: T) -> T
```