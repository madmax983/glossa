# LeetCode Examples in ΓΛΩΣΣΑ

Demonstrations of LeetCode-style algorithmic patterns using GLOSSA's collection features: HashMap (χάρτης), HashSet (σύνολον), and mutable bindings (μετά).

## Running the Demo

```bash
cargo run -- examples/leetcode_working.γλ
```

## Patterns Demonstrated

### 1. Two Sum (HashMap Complement Lookup)
**Difficulty:** Easy/Medium
**Key Technique:** Store seen numbers with their indices

```glossa
χ νέον χάρτης ἔστω.        // Create HashMap
χ 2 0 τίθησι.              // map.insert(2, 0)
χ 7 1 τίθησι.              // map.insert(7, 1)
2 ἐν χ? λέγε.              // map.contains_key(&2) → true
```

**Algorithm:**
- For each number, check if its complement exists in the map
- If yes: found the pair!
- If no: add current number → index to map

### 2. Contains Duplicate (HashSet)
**Difficulty:** Easy
**Key Technique:** Track unique elements

```glossa
σ νέον σύνολον ἔστω.       // Create HashSet
σ 1 τίθησι.                // set.insert(1)
σ 2 τίθησι.                // set.insert(2)
1 ἐν σ? λέγε.              // set.contains(&1) → true
5 ἐν σ? λέγε.              // set.contains(&5) → false
```

**Algorithm:**
- For each element, check if it's already in the set
- If yes: found a duplicate!
- If no: add to set

### 3. Character Frequency Counter (HashMap)
**Difficulty:** Easy/Medium (used in Valid Anagram, etc.)
**Key Technique:** Count occurrences

```glossa
φ νέον χάρτης ἔστω.        // Create frequency map
φ «h» 1 τίθησι.            // freq['h'] = 1
φ «l» 2 τίθησι.            // freq['l'] = 2
«l» ἐν φ? λέγε.            // freq.contains_key("l") → true
```

**Algorithm:**
- For each character, increment its count in the map
- Useful for: anagram detection, string comparison

### 4. Mutable Counter Pattern
**Difficulty:** Basic (used everywhere)
**Key Technique:** Track state changes

```glossa
μετά κ 0 ἔστω.             // let mut counter = 0
κ λέγε.                    // println!("{}", counter) → 0
κ 1 γίγνεται.              // counter = 1
κ λέγε.                    // println!("{}", counter) → 1
```

**Use cases:**
- Loop counters
- Running totals
- State tracking

### 5. Array Indexing
**Difficulty:** Basic
**Key Techniques:** Ordinal indexing and bracket notation

```glossa
ν [2, 7, 11, 15] ἔστω.     // let nums = vec![2, 7, 11, 15]
ν πρῶτον λέγε.             // nums[0] → 2
ν δεύτερον λέγε.           // nums[1] → 7
ν[2] λέγε.                 // nums[2] → 11
ν μῆκος λέγε.              // nums.len() → 4
```

**Patterns:**
- πρῶτον (first) → index 0
- δεύτερον (second) → index 1
- τρίτον (third) → index 2

### 6. Sliding Window Setup
**Difficulty:** Medium
**Key Technique:** Two pointers with mutable state

```glossa
μετά ἀριστερά 0 ἔστω.      // let mut left = 0
μετά δεξιά 0 ἔστω.         // let mut right = 0
μετά μέγιστος 0 ἔστω.      // let mut max_len = 0

ἀριστερά 1 γίγνεται.       // left = 1
δεξιά 3 γίγνεται.          // right = 3
μέγιστος 10 γίγνεται.      // max_len = 10
```

**Use cases:**
- Longest substring problems
- Subarray problems
- Window-based algorithms

## Grammar Reference

### HashMap (χάρτης - "map/chart")
```glossa
χ νέον χάρτης ἔστω.              // Create
χ «key» «value» τίθησι.          // Insert
«key» ἐν χ?                      // Contains key
```

### HashSet (σύνολον - "set/collection")
```glossa
σ νέον σύνολον ἔστω.             // Create
σ 42 τίθησι.                     // Insert
42 ἐν σ?                         // Contains
```

### Mutable Variables (μετά - "changeable")
```glossa
μετά ξ 5 ἔστω.                   // let mut xi = 5
ξ 10 γίγνεται.                   // xi = 10
```

**IMPORTANT:** Use numeric literals (0, 1, 5) instead of Greek number words (μηδέν, ἓν, πέντε) for assignment values to avoid parsing ambiguity.

### Arrays
```glossa
ξ [1, 2, 3] ἔστω.                // let xi = vec![1, 2, 3]
ξ πρῶτον                         // xi[0]
ξ[0]                             // xi[0] (bracket notation)
ξ μῆκος                          // xi.len()
```

## LeetCode Problem Mapping

| Problem | Pattern | GLOSSA Features |
|---------|---------|-----------------|
| Two Sum | HashMap complement | χάρτης, τίθησι, ἐν...? |
| Contains Duplicate | HashSet uniqueness | σύνολον, τίθησι, ἐν...? |
| Valid Anagram | Frequency counter | χάρτης, τίθησι |
| Longest Substring | Sliding window + HashSet | μετά, γίγνεται, σύνολον |
| Subarray Sum | Prefix sum + mutable | μετά, γίγνεται |
| Group Anagrams | Frequency as key | χάρτης, string ops |

## Next Steps

With these building blocks, you can implement most array/hashmap LeetCode problems! The main missing pieces for full solutions are:
- Loops (διὰ for iteration, ἕως for while)
- Conditionals (εἰ...ᾖ for if statements)
- Functions (for reusable logic)

See `docs/reference/` for full language documentation.
