# 18. Option and Result

## 18.1 Option Type

Options represent values that may or may not exist. They use the Greek vocabulary of "something" and "nothing":

### Some — τί ("something")

```glossa
ξ τι πέντε ἔστω.
"x something five let-be."
// → let x = Some(5);

ψ τι «χαῖρε» ἔστω.
"y something 'hello' let-be."
// → let y = Some("hello");
```

### None — οὐδέν ("nothing")

```glossa
ξ ουδεν ἔστω.
"x nothing let-be."
// → let x: Option<i64> = None;
```

## 18.2 Result Type

Results represent computations that may succeed or fail:

### Ok — ἐπιτυχία ("success")

```glossa
ξ επιτυχια δέκα ἔστω.
"x success ten let-be."
// → let x = Ok(10);
```

### Err — σφάλμα ("error/mistake")

```glossa
ξ σφαλμα «πρόβλημα» ἔστω.
"x error 'problem' let-be."
// → let x = Err("problem");
```

## 18.3 Unwrap — `!` operator

The **`!`** suffix performs confident extraction (panics if None/Err):

```glossa
ξ τι πέντε ἔστω.
ψ ξ! ἔστω.
"y x! let-be."
// → let y = x.unwrap();

ξ! λέγε.
"x! say."
// → println!("{}", x.unwrap());
```

## 18.4 Propagation — `;` operator

Ending a statement with **`;`** (ASCII semicolon) instead of **`.`** propagates None/Err upward (equivalent to Rust's `?` operator):

```glossa
τιμη τι πεντε εστω;
"value something five let-be;"
// → let timee = Some(5)?;
// If None, returns early from the enclosing function
```

## 18.5 Summary

| ΓΛΩΣΣΑ | Meaning | Rust |
|---------|---------|------|
| `τι value` | something | `Some(value)` |
| `ουδεν` | nothing | `None` |
| `επιτυχια value` | success | `Ok(value)` |
| `σφαλμα value` | error | `Err(value)` |
| `expr!` | confident extract | `expr.unwrap()` |
| `stmt;` | propagate | `stmt?` |