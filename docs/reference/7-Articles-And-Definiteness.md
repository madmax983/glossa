# 7. Articles and Definiteness

## 7.1 The Article

Greek has a definite article that declines:

| Case | Masc | Fem | Neut |
|------|------|-----|------|
| Nom | ὁ | ἡ | τό |
| Gen | τοῦ | τῆς | τοῦ |
| Dat | τῷ | τῇ | τῷ |
| Acc | τόν | τήν | τό |

## 7.2 Programming Meaning

| Form | Meaning | Programming Use |
|------|---------|-----------------|
| **Bare noun** | Indefinite "a/an" | New allocation |
| **With article** | Definite "the" | Reference to existing |
| **οὗτος** | "this" | `self`, current scope |
| **ἐκεῖνος** | "that" | Outer scope, captured |

### Examples

```glossa
// BARE — New allocation
χρήστης.
// → let user = User::new()

// ARTICLE — Existing reference
ὁ χρήστης.
"the user"
// → &user

// THIS — Self reference
τούτου ὄνομα.
"of-this name"
// → self.name

// THAT — Closure capture
ἐκείνου ὄνομα.
"of-that name"
// → captured.name
```