# 8. Statements and Expressions

## 8.1 Statement Structure

A statement is one or more expressions terminated by a period:

```glossa
expression.                      // Single expression
expression · expression.         // Chained expressions
```

The middle dot (`·`) chains expressions like a semicolon.

## 8.2 Expression Evaluation

Expressions are assembled from slots, not parsed positionally:

```glossa
χρήστου ὄνομα λέγε.
```

Slots filled:
- Genitive: χρήστου → possession
- Nominative: ὄνομα → subject/value  
- Verb: λέγω (imperative) → print

Result: `print(user.name)`

## 8.3 Queries

Statements ending with `;` (Greek question mark) are queries:

```glossa
χρήστης ὑπάρχει;
"user exists?"
// → user.exists()  // returns bool
```