# 14. Types (Εἶδος)

## 14.1 Struct Definition

The genitive + εἶδος pattern:

```glossa
Χρήστου εἶδος·
    ὄνομα λόγος·
    ἡλικία ἀριθμός.

"Of-User form:
    name [is] string;
    age [is] number."
```

→

```rust
struct Chrestes {
    onoma: String,
    helikia: i64,
}
```

## 14.2 Field Modifiers

```glossa
Ἀποτελέσματος εἶδος·
    τιμὴ ἀριθμὸς ἴσως·      // Option<i64>
    σφάλματα λόγοι πολλά·   // Vec<String>
    μετρητὴς ἀριθμὸς μετά.  // mut i64

// ἴσως = maybe → Option<T>
// πολλά = many → Vec<T>
// μετά = mutable → mut
```

## 14.3 Instantiation

```glossa
// Constructor call
χρήστης Χρήστης.νέος «Σωκράτης» ἑβδομήκοντα.
// → let user = User::new("Socrates", 70);

// Literal syntax
χρήστης Χρήστης· ὄνομα «Σωκράτης» · ἡλικία ἑβδομήκοντα.
// → let user = User { name: "Socrates", age: 70 };
```

## 14.4 Field Access

Genitive chain:

```glossa
χρήστου ὄνομα
"of-user name"
// → user.name

χρήστου ἡλικίας διπλάσιον
"of-user of-age double"
// → user.age * 2
```