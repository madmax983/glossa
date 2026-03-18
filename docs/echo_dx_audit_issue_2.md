# 🗣️ Echo: Getting Started examples and errors are broken

🤦 **The Confusion:**
I was following the Troubleshooting guide in the `README.md` to see what `Οὐκ οἶδα τὸ ὄνομα` means. It says: "Undefined variable". I tried to recreate this with:
```glossa
ἄγνωστον λέγε.
```
But the compiler just succeeded and printed an empty line!

I also tried the "Double Subject" error (`Διπλοῦν ὑποκείμενον`):
```glossa
ὁ ἄνθρωπος ὁ θεός λέγει.
```
It compiled successfully without errors!

🕵️ **The Reality:**
It seems like undefined variables silently pass because of how `λέγε` consumes undefined objects or how adjectives are handled. And the parser/analyzer doesn't catch double nominatives, completely ignoring the `Διπλοῦν ὑποκείμενον` error it advertises.

💡 **The Fix:**
The compiler needs to correctly fail and emit `Οὐκ οἶδα τὸ ὄνομα` when an undefined variable is referenced. It should also catch when two nominative subjects are supplied and emit `Διπλοῦν ὑποκείμενον`. The documentation promises these errors but the compiler completely ignores them!
