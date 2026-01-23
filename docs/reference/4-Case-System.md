# 4. The Case System

## 4.1 Five Cases

| Case | Typical Endings | Semantic Role | Programming Meaning |
|------|-----------------|---------------|---------------------|
| **Nominative** | -ος, -η, -ον, -α | Subject, "doer" | Value, LHS of binding, return |
| **Genitive** | -ου, -ης, -ου, -ων | "of X", possession | Property access, `.field` |
| **Dative** | -ῳ, -ῃ, -ῳ, -οις | "to/for X", recipient | Method receiver, channel, callback |
| **Accusative** | -ον, -ην, -ον, -ους | Direct object | Argument, input, consumed value |
| **Vocative** | -ε, -η, -ον | Direct address | Errors, debug, REPL |

## 4.2 Case Usage Examples

### Nominative — Subject and Values

```glossa
ἀποτέλεσμα πέντε.
"result [nom] five"
// → result = 5

χρήστης γράφει.
"user [nom] writes"
// → user.write() — user is the subject
```

### Genitive — Property Access

```glossa
χρήστου ὄνομα
"of-user [gen] name [nom]"
// → user.name

χρήστου προφίλου εἰκόνος μέγεθος
"of-user of-profile of-image size"
// → user.profile.image.size
```

### Dative — Recipient / Target

```glossa
λίστῃ στοιχεῖον ὠθεῖ.
"to-list [dat] element [acc] pushes"
// → list.push(element)

χρήστῃ μήνυμα πέμπει.
"to-user [dat] message [acc] sends"
// → user.send(message)
```

### Accusative — Direct Object

```glossa
δεδομένα γράφει.
"data [acc] writes"
// → write(data)

ἀριθμὸν διπλασιάζει.
"number [acc] doubles"
// → double(number)
```

### Vocative — Errors and Debug

```glossa
ὦ χρῆστα, σφάλμα!
"O user [voc], error!"
// → panic!("error") or debug output
```

## 4.3 Declension Tables

### Second Declension (Masculine -ος)

| Case | Singular | Plural |
|------|----------|--------|
| Nom | χρήστ**ος** | χρήστ**οι** |
| Gen | χρήστ**ου** | χρήστ**ων** |
| Dat | χρήστ**ῳ** | χρήστ**οις** |
| Acc | χρήστ**ον** | χρήστ**ους** |
| Voc | χρήστ**ε** | χρήστ**οι** |

### Second Declension (Neuter -ον)

| Case | Singular | Plural |
|------|----------|--------|
| Nom | στοιχεῖ**ον** | στοιχεῖ**α** |
| Gen | στοιχεί**ου** | στοιχεί**ων** |
| Dat | στοιχεί**ῳ** | στοιχεί**οις** |
| Acc | στοιχεῖ**ον** | στοιχεῖ**α** |

### First Declension (Feminine -η)

| Case | Singular | Plural |
|------|----------|--------|
| Nom | λίστ**η** | λίστ**αι** |
| Gen | λίστ**ης** | λιστ**ῶν** |
| Dat | λίστ**ῃ** | λίστ**αις** |
| Acc | λίστ**ην** | λίστ**ας** |

### Third Declension (Neuter -μα)

| Case | Singular | Plural |
|------|----------|--------|
| Nom | ὄνο**μα** | ὀνό**ματα** |
| Gen | ὀνό**ματος** | ὀνο**μάτων** |
| Dat | ὀνό**ματι** | ὀνό**μασι** |
| Acc | ὄνο**μα** | ὀνό**ματα** |