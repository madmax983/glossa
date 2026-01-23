# 12. Collections

## 12.1 Arrays (Πίναξ)

### Literals

```glossa
// Array literal
[εἷς, δύο, τρεῖς] ἔστω ἀριθμοί.
// → let numbers = [1, 2, 3];

// Empty array with type
πίναξ ἀριθμῶν κενός ἔστω.
"array of-numbers empty let-be"
// → let arr: Vec<i64> = vec![];
```

### Indexing

```glossa
// Access by index (genitive of position)
πίνακος τρίτου στοιχεῖον
"of-array of-third element"
// → array[2]

// Or with ἐν (in/at)
στοιχεῖον ἐν πίνακι τρίτον
"element in array third"
// → array[2]
```

### Methods

```glossa
// Push (dative recipient + accusative item)
πίνακι στοιχεῖον ὠθεῖ.
"to-array element pushes"
// → array.push(element)

// Pop (middle voice - array acts on itself)
πίναξ ἕλκεται.
"array pulls-itself"
// → array.pop()

// Length
πίνακος μῆκος
"of-array length"
// → array.len()

// Is empty
πίναξ κενός;
"array empty?"
// → array.is_empty()
```

## 12.2 HashMap (Χάρτης)

### Creation

```glossa
χάρτης κενὸς ἔστω.
"map empty let-be"
// → let map = HashMap::new();
```

### Set

```glossa
χάρτῃ κλειδὶ τιμὴν τίθησι.
"to-map with-key value places"
// → map.insert(key, value)
```

### Get

```glossa
χάρτου κλειδὸς τιμή
"of-map of-key value"
// → map.get(&key)

// Returns Option, so often with optative:
χάρτου κλειδὸς τιμὴ εὑρεθείη.
"of-map of-key value might-be-found"
// → map.get(&key)  // Option<V>
```

### Contains

```glossa
κλεὶς ἐν χάρτῃ;
"key in map?"
// → map.contains_key(&key)
```

## 12.3 Set (Σύνολον)

```glossa
// Create
σύνολον κενὸν ἔστω.

// Add
συνόλῳ στοιχεῖον τίθησι.
"to-set element places"
// → set.insert(element)

// Contains
στοιχεῖον ἐν συνόλῳ;
"element in set?"
// → set.contains(&element)
```

## 12.4 String (Λόγος)

```glossa
// Length
λόγου μῆκος
// → string.len()

// Contains
«κόσμε» ἐν λόγῳ;
// → string.contains("κόσμε")

// Split
λόγος κατὰ « » σχίζεται.
"string according-to ' ' splits-itself"
// → string.split(' ')

// Join
λόγοι κατὰ «, » ἑνοῦνται.
"strings according-to ', ' unite-themselves"
// → strings.join(", ")
```