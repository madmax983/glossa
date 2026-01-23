# 17. Participles as Lambdas

## 17.1 The Insight

Greek participles are **verbal adjectives** — they describe nouns while carrying verbal aspect and voice. This maps perfectly to lambdas/closures.

| Participle | Form | Meaning | Lambda Use |
|------------|------|---------|------------|
| Present Active | γράφων | "writing" | `\|x\| x.write()` |
| Present Middle | γραφόμενος | "writing itself" | `\|x\| x.self_write()` |
| Aorist Active | γράψας | "having written" | `\|x\| { write(x); }` |
| Perfect Passive | γεγραμμένος | "having been written" | cached/memoized |

## 17.2 Map with Participle

```glossa
λίστης στοιχεῖα διπλασιαζόμενα.
"of-list elements being-doubled"
// → list.iter().map(|x| x * 2)

ἀριθμοὶ τετραγωνισμένοι.
"numbers squared"
// → numbers.map(|n| n * n)
```

## 17.3 Filter with Participle

```glossa
λίστης στοιχεῖα μηδενὸς μείζονα.
"of-list elements than-zero greater"
// → list.iter().filter(|x| x > 0)

χρῆσται ἐνεργοί.
"users active"
// → users.filter(|u| u.is_active())
```

## 17.4 Combined Operations

```glossa
λίστης στοιχεῖα μηδενὸς μείζονα διπλασιαζόμενα.
"of-list elements than-zero greater being-doubled"
// → list.iter().filter(|x| x > 0).map(|x| x * 2)
```

## 17.5 Participle Declension (Present Active -ων)

| Case | Masc | Fem | Neut |
|------|------|-----|------|
| Nom | γράφων | γράφουσα | γράφον |
| Gen | γράφοντος | γραφούσης | γράφοντος |
| Dat | γράφοντι | γραφούσῃ | γράφοντι |
| Acc | γράφοντα | γράφουσαν | γράφον |