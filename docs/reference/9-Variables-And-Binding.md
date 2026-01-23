# 9. Variables and Binding

## 9.1 Immutable Binding

The imperative of εἰμί ("to be") creates bindings:

```glossa
ξ πέντε ἔστω.
"x five let-it-be"
// → let x = 5;

ὄνομα «Σωκράτης» ἔστω.
"name 'Socrates' let-it-be"
// → let name = "Socrates";
```

## 9.2 Mutable Binding

Use μεταβάλλεται ("changes") or μετά prefix:

```glossa
ξ πέντε μεταβάλλεται.
"x five changes"
// → let mut x = 5;

// Or with explicit mutability marker
μετὰ ξ πέντε ἔστω.
"mutable x five let-be"
// → let mut x = 5;
```

## 9.3 Assignment

Middle voice indicates self-change:

```glossa
ξ δέκα γίγνεται.
"x ten becomes"
// → x = 10;
```

## 9.4 Shadowing

New binding with same name shadows:

```glossa
ξ πέντε ἔστω.
ξ «hello» ἔστω.    // Shadows with new type
```
