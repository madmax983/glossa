# 3. Morphological System

## 3.1 How Parsing Works

The parser produces a flat list of tokens. The **morphological analyzer** examines each Greek word and extracts:

```rust
struct MorphAnalysis {
    lemma: String,           // Dictionary form
    part_of_speech: PartOfSpeech,
    case: Option,      // For nominals
    number: Option,
    gender: Option,
    person: Option,  // For verbs
    tense: Option,
    aspect: Option,
    mood: Option,
    voice: Option,
}
```

## 3.2 The Slot-Based Assembler

The **assembler** routes words to semantic slots based on their morphological features:

```
Input: γράφει χρήστης δεδομένα.

Morphological Analysis:
  γράφει  → verb, present, active, indicative, 3rd singular
  χρήστης → noun, nominative, singular, masculine
  δεδομένα → noun, accusative, plural, neuter

Slot Assignment:
  nominative → subject slot: χρήστης
  accusative → object slot: δεδομένα
  verb → verb slot: γράφω

Result:
  Subject: χρήστης
  Verb: γράφω (present, active)
  Object: δεδομένα
```

Word order is irrelevant. The slots are filled by case, not position.