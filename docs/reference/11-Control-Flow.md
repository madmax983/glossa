# 11. Control Flow

## 11.1 Conditionals (Subjunctive)

The subjunctive mood creates conditionals:

```glossa
// Simple if
εἰ ξ πέντε μεῖζον ᾖ, ἀποτέλεσμα ἀληθές.
"if x than-five greater be-[subj], result true"
// → if x > 5 { result = true }

// If-else
εἰ ξ πέντε μεῖζον ᾖ, ἀποτέλεσμα ἀληθές · εἰ δὲ μή, ψεῦδος.
"if x than-five greater be-[subj], result true; if but not, false"
// → if x > 5 { true } else { false }

// Else-if chain
εἰ ξ μηδὲν ᾖ, «μηδέν» · εἰ ξ ἓν ᾖ, «ἕν» · εἰ δὲ μή, «ἄλλο».
"if x zero be, 'zero'; if x one be, 'one'; if but not, 'other'"
// → if x == 0 { "zero" } else if x == 1 { "one" } else { "other" }
```

### Conditional Particles

| Greek | Usage |
|-------|-------|
| εἰ | if (with indicative or optative) |
| ἐάν / ἤν / ἄν | if (with subjunctive) |
| εἰ δὲ μή | else (literally "if but not") |
| ἄλλως | otherwise |

## 11.2 Pattern Matching

Use κατά + accusative for matching:

```glossa
κατὰ τιμήν·
    μηδὲν ᾖ, «μηδέν»·
    ἓν ᾖ, «ἕν»·
    ἄλλο ᾖ, «ἄλλο».

"according-to value:
    zero be, 'zero';
    one be, 'one';
    other be, 'other'"

// → match value { 0 => "zero", 1 => "one", _ => "other" }
```

## 11.3 Loops

### While Loop (ἕως + Subjunctive)

```glossa
ἕως ξ μηδενὸς μεῖζον ᾖ, ξ μειοῦται.
"while x than-zero greater be-[subj], x decreases-[middle]"
// → while x > 0 { x -= 1 }
```

### For Loop (Iteration with Present Aspect)

```glossa
λίστης στοιχεῖα λέγει.
"of-list elements says-[present]"
// → for elem in list { print(elem) }

// With explicit δία (through)
διὰ στοιχείων λίστης, στοιχεῖον λέγε.
"through elements of-list, element say"
// → for elem in list { print(elem) }
```

### Range Loop

```glossa
ἀπὸ μηδενὸς μέχρι δέκα, ι λέγε.
"from zero until ten, i say"
// → for i in 0..10 { print(i) }

ἀπὸ μηδενὸς ἕως δέκα, ι λέγε.
"from zero to ten [inclusive], i say"
// → for i in 0..=10 { print(i) }
```

## 11.4 Loop Control

```glossa
παῦε.           // break (imperative of παύω)
συνέχιζε.       // continue (imperative of συνεχίζω)
```

## 11.5 Early Return

```glossa
δίδου τιμήν.    // return value (imperative)
διδόσθω.        // let it be returned (imperative passive)
```