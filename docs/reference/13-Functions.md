# 13. Functions

## 13.1 Function Definition

Functions are defined with **ὁρίζειν** ("to define/delimit"). Parameters use the **dative case** marker **τῷ** ("to/for"):

```glossa
// Simple function — no parameters
χαιρετισμος ὁρίζειν· «χαῖρε» λέγε.
"greeting to-define: 'hello' say."
// → fn chairetismos() { println!("hello"); }

// Function with one parameter
διπλασιασμος ὁρίζειν τῷ ξ· δός ξ δύο γινόμενον.
"doubling to-define for x: give x two product."
// → fn diplasiasmos(xi: i64) -> i64 { return xi * 2; }
```

### With Type Annotations

Type annotations use the **genitive** (type "of" the parameter):

```glossa
προσθεσις ὁρίζειν τῷ ξ ἀριθμοῦ τῷ ψ ἀριθμοῦ· δός ξ ψ ἄθροισμα.
"addition to-define for x of-number for y of-number: give x y sum."
// → fn prosthesis(xi: i64, psi: i64) -> i64 { return xi + psi; }
```

## 13.2 Return Statements

The imperative **δός** ("give!") serves as `return`:

```glossa
διπλασιασμος ὁρίζειν τῷ ξ· δός ξ δύο γινόμενον.
// → fn diplasiasmos(xi: i64) -> i64 { return xi * 2; }
```

Return types are inferred from the expression given to δός. When parameters have type annotations, the return type is explicit in generated code.

## 13.3 Function Calls

Functions are called by name with arguments. Bind the result with **ἔστω**:

```glossa
ἀποτελεσμα προσθεσις πέντε τρία ἔστω.
"result addition five three let-be."
// → let apotelasma = prosthesis(5, 3);
```

### Nested Calls

Parentheses group nested function calls:

```glossa
ψ διπλασιασμος (διπλασιασμος πέντε) ἔστω.
"y doubling (doubling five) let-be."
// → let psi = diplasiasmos(diplasiasmos(5));
```

## 13.4 Multiple Parameters

Parameters are introduced with repeated **τῷ** markers:

```glossa
προσθεσις ὁρίζειν τῷ ξ τῷ ψ· δός ξ ψ ἄθροισμα.
"addition to-define for x for y: give x y sum."
// → fn prosthesis(xi: i64, psi: i64) -> i64 { return xi + psi; }
```

## 13.5 Local Variables

Functions can define local bindings with **ἔστω**:

```glossa
αυξησις ὁρίζειν τῷ ξ·
    τοπικον ξ ἓν ἄθροισμα ἔστω·
    δός τοπικον.

"increment to-define for x:
    local x one sum let-be;
    give local."
// → fn auxesis(xi: i64) -> i64 { let topikon = xi + 1; return topikon; }
```