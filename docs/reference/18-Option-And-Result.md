# 18. Option and Result

## 18.1 Option Type (Optative Mood)

The optative mood ("might be") naturally expresses `Option<T>`:

```glossa
// Declaration
τιμὴ εὑρεθείη.
"value might-be-found-[optative]"
// → let value: Option<T> = find();

// Check and unwrap
τιμὴ εὑρεθείη·
    ὑπάρχουσα, τιμὴν χρῶ·
    οὐδὲν οὖσα, σφάλμα.

"value might-be-found:
    existing, value use;
    nothing being, error."

// → match value { Some(v) => use(v), None => error() }
```

## 18.2 Some and None

```glossa
τί τιμή         // Some(value)  — τί = "something"
οὐδέν           // None         — "nothing"
```

## 18.3 Result Type

```glossa
// Result declaration
ἀποτέλεσμα ἢ ἐπιτυχία ἢ σφάλμα.
"result either success or error"

// Pattern match
κατὰ ἀποτέλεσμα·
    ἐπιτυχία ᾖ, τιμὴν χρῶ·
    σφάλμα ᾖ, σφάλμα λέγε.
```

## 18.4 Error Handling

```glossa
// Propagate with ?
τιμὴ εὑρεθείη;    // The ; propagates None/Err

// Unwrap (panics)
τιμὴ εὑρέθη!      // Indicative + ! = unwrap
```