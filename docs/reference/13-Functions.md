# 13. Functions

## 13.1 Function Definition

Functions use the infinitive or are introduced with ἔργον:

```glossa
// Simple function
διπλασιασμὸς ἔργον ἀριθμῷ·
    ἀριθμὸς δύο πολλαπλασιασθείς.

"doubling [is] a work for-number:
    number two multiplied"

// → fn double(n: i64) -> i64 { n * 2 }
```

### With Explicit Types

```glossa
διπλασιασμὸς ἔργον· ἀριθμῷ ἀριθμὸν δίδωσι·
    ἀριθμὸς δύο γινόμενον.

"doubling [is] a work: for-number gives number:
    number two product"

// → fn double(n: i64) -> i64 { n * 2 }
```

## 13.2 Function Call

```glossa
πέντε διπλασιάζεται.
"five is-doubled"
// → double(5)

// Or with explicit object
πέντε διπλασιασμός.
"five doubling"
// → double(5)
```

## 13.3 Multiple Parameters

Parameters in dative case:

```glossa
πρόσθεσις ἔργον· αῳ βῳ ἀριθμὸν δίδωσι·
    α β ἄθροισμα.

"addition [is] a work: for-a for-b gives number:
    of-a of-b sum"

// → fn add(a: i64, b: i64) -> i64 { a + b }
```

## 13.4 No Return Value

```glossa
ἐκτύπωσις ἔργον· λόγῳ·
    λόγον λέγε.

"printing [is] a work: for-string:
    string say"

// → fn print(s: &str) { println!("{}", s); }
```