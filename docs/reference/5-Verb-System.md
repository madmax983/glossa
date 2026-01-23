# 5. The Verb System

## 5.1 Overview

Greek verbs encode five dimensions:

| Dimension | Values | Programming Meaning |
|-----------|--------|---------------------|
| **Person** | 1st, 2nd, 3rd | Who acts |
| **Number** | Sing, Dual, Plural | How many |
| **Tense/Aspect** | Present, Aorist, Perfect, Future | Ownership semantics |
| **Mood** | Indicative, Subjunctive, Optative, Imperative | Control flow |
| **Voice** | Active, Middle, Passive | Self-reference |

## 5.2 Aspect and Ownership

This is the core innovation: **tense/aspect maps to ownership semantics**.

| Aspect | Greek | Meaning | Ownership | Rust Equivalent |
|--------|-------|---------|-----------|-----------------|
| **Present** | γράφει | Ongoing, repeated | Borrow | `&data` |
| **Aorist** | ἔγραψε | One-shot, completed | Move | `data` (consumed) |
| **Perfect** | γέγραφε | Completed with result | Cached | `.clone()` or memoized |
| **Future** | γράψει | Will happen | Lazy/Promise | `async`, `Future<T>` |

### Examples

```glossa
// PRESENT — Borrow (data still valid after)
δεδομένα λόγγῳ γράφει.
"data to-logger writes-[present]"
// → logger.write(&data)

// AORIST — Move (data consumed)
δεδομένα λόγγῳ ἔγραψε.
"data to-logger wrote-[aorist]"
// → logger.write(data)  // data no longer valid

// PERFECT — Cached/memoized
δεδομένα γέγραφε.
"data has-been-written-[perfect]"
// → result is cached, won't recompute

// FUTURE — Async/lazy
δεδομένα γράψει.
"data will-write-[future]"
// → Future<()> or lazy evaluation
```

## 5.3 Voice

| Voice | Form | Meaning | Programming Use |
|-------|------|---------|-----------------|
| **Active** | γράφει | Subject acts on object | Normal operations |
| **Middle** | γράφεται | Subject acts on itself | Self-mutation, `&mut self` |
| **Passive** | γράφεται | Subject receives action | Event handlers, callbacks |

### Examples

```glossa
// ACTIVE — Normal operation
χρήστης δεδομένα σῴζει.
"user data saves-[active]"
// → user.save(data)

// MIDDLE — Self-mutation
χρήστης σῴζεται.
"user saves-itself-[middle]"
// → user.serialize() or self.save()

// PASSIVE — Event handling
δεδομένα σῴζεται, τότε λόγγῳ.
"data is-saved-[passive], then to-logger"
// → on_save(data, |_| logger.log())
```

## 5.4 Mood

| Mood | Meaning | Endings (3sg) | Programming Use |
|------|---------|---------------|-----------------|
| **Indicative** | Definite fact | -ει | Normal execution |
| **Subjunctive** | Hypothetical | -ῃ | Conditionals (`if`) |
| **Optative** | Possibility | -οι | `Option<T>`, `Maybe` |
| **Imperative** | Command | -ε, -έτω | Side effects, REPL |

### Examples

```glossa
// INDICATIVE — Definite
χρήστης ὑπάρχει.
"user exists-[indic]"
// → assert!(user.exists())

// SUBJUNCTIVE — Conditional
χρήστης ὑπάρχῃ, ...
"user exist-[subj], ..."
// → if user.exists() { ... }

// OPTATIVE — Optional
χρήστης ὑπάρχοι.
"user might-exist-[opt]"
// → user: Option<User>

// IMPERATIVE — Command
γράφε!
"write-[imper]!"
// → immediate execution, side effect
```

## 5.5 Conjugation Tables

### Present Active Indicative (γράφω "I write")

| Person | Singular | Plural |
|--------|----------|--------|
| 1st | γράφ**ω** | γράφ**ομεν** |
| 2nd | γράφ**εις** | γράφ**ετε** |
| 3rd | γράφ**ει** | γράφ**ουσι** |

### Aorist Active Indicative (ἔγραψα "I wrote")

| Person | Singular | Plural |
|--------|----------|--------|
| 1st | ἔγραψ**α** | ἐγράψ**αμεν** |
| 2nd | ἔγραψ**ας** | ἐγράψ**ατε** |
| 3rd | ἔγραψ**ε(ν)** | ἔγραψ**αν** |

### Present Middle/Passive (γράφομαι)

| Person | Singular | Plural |
|--------|----------|--------|
| 1st | γράφ**ομαι** | γραφ**όμεθα** |
| 2nd | γράφ**ῃ** | γράφ**εσθε** |
| 3rd | γράφ**εται** | γράφ**ονται** |

### Present Subjunctive (γράφω — same stem, long vowels)

| Person | Singular | Plural |
|--------|----------|--------|
| 1st | γράφ**ω** | γράφ**ωμεν** |
| 2nd | γράφ**ῃς** | γράφ**ητε** |
| 3rd | γράφ**ῃ** | γράφ**ωσι** |

### Present Optative (γράφοιμι)

| Person | Singular | Plural |
|--------|----------|--------|
| 1st | γράφ**οιμι** | γράφ**οιμεν** |
| 2nd | γράφ**οις** | γράφ**οιτε** |
| 3rd | γράφ**οι** | γράφ**οιεν** |

### Imperative

| Person | Singular | Plural |
|--------|----------|--------|
| 2nd | γράφ**ε** | γράφ**ετε** |
| 3rd | γραφ**έτω** | γραφ**όντων** |