# 6. Number

## 6.1 Three Numbers

Greek has three grammatical numbers:

| Number | Meaning | Programming Use |
|--------|---------|-----------------|
| **Singular** | One | Single value |
| **Dual** | Exactly two | Tuples, pairs, key-value, swap |
| **Plural** | Many | Arrays, collections, variadics |

## 6.2 Dual for Pairs

The dual is perfect for two-element operations:

```glossa
// Swap two values
α β ἀλλάσσεσθον.
"alpha beta swap-[dual-middle]"
// → (a, b) = (b, a)

// Key-value pair
κλειδὸς τιμῆς ζεῦγος
"of-key of-value pair-[dual]"
// → (key, value)

// Binary operation
α β προστίθεσθον.
"alpha beta add-[dual]"
// → a + b
```

## 6.3 Plural for Collections

```glossa
// Iteration (plural subject)
στοιχεῖα λέγει.
"elements-[plural] says"
// → for elem in elements { print(elem) }

// Variadic
ἀριθμοὺς ἀθροίζει.
"numbers-[plural-acc] sums"
// → sum(numbers)
```
