# 1. Philosophy

## 1.1 The Core Thesis

**Inflection over position.** The grammatical form of words—their case endings, verb conjugations, and agreement patterns—determines their semantic role in a statement. Word order is free.

```glossa
// All four are identical:
χρήστης δεδομένα γράφει.    // SOV
γράφει χρήστης δεδομένα.    // VSO
δεδομένα χρήστης γράφει.    // OSV
γράφει δεδομένα χρήστης.    // VOS

// The parser doesn't care about order.
// It reads the endings:
//   χρήστης (-ης nominative) → subject
//   δεδομένα (-α accusative) → object
//   γράφει (-ει 3rd sing) → verb
```

## 1.2 Why Ancient Greek?

Ancient Greek has the richest morphological system of any well-documented language:

| Feature | What It Encodes | Programming Use |
|---------|-----------------|-----------------|
| 5 cases | Semantic roles | Subject, object, property, recipient, error |
| 3 voices | Agency direction | Normal ops, self-mutation, event handling |
| 4 moods | Certainty level | Execution, conditionals, optionals, commands |
| 6 tenses/aspects | Time and completion | Borrow, move, cache, async |
| 3 numbers | Quantity | Single, pair, collection |
| 3 genders | Type categories | Agents, containers, data |

## 1.3 Design Principles

1. **No keywords for structure** — Grammar IS the syntax
2. **Agreement as type-checking** — Gender/number/case matching catches errors
3. **Aspect as ownership** — Tense encodes borrow vs move vs copy
4. **Voice as self-reference** — Middle voice for mutation methods
5. **Mood as control flow** — Subjunctive for `if`, optative for `Option`
6. **Participles as lambdas** — Verbal adjectives for inline functions