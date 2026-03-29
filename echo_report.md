# 🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:**
When trying to run the quickstart code from the README's "Chapter 3: The Structure (Types)", the compiler throws a mysterious syntax error: `expected statement_end` right at the start of creating the User instance.

🕵️ **The Reality:**
In `README.md`, Chapter 3 ends the struct definition like this:
```glossa
εἶδος Χρήστης ὁρίζειν {
    ὄνομα ὀνόματος.    // String
    ἡλικία ἀριθμοῦ. // i64
}
```
But the language actually requires a dot (`.`) at the end of the `εἶδος` definition block, just like `examples/quickstart.γλ` has:
```glossa
εἶδος Χρήστης ὁρίζειν {
    ὄνομα ὀνόματος.    // String
    ἡλικία ἀριθμοῦ. // i64
}.
```

💡 **The Fix:**
Update the `README.md` code snippet in "Chapter 3: The Structure (Types)" to include the missing dot (`.`) after the closing brace `}` so users can actually copy-paste the example without getting confusing compiler errors.