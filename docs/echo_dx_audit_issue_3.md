# 🗣️ Echo: Getting Started example is broken

## 🤦 **The Confusion:**
Tried to run the `story_demo` example from the `nova` feature logic:
`εἶδος Ἱστορία ὁρίζειν { τίτλος ὀνόματος. }. ἱστορία νέον Ἱστορία «Ἡ Ὀδύσσεια» ἔστω. τίτλος λέγε.`
I expected it to print "Ἡ Ὀδύσσεια". Instead, `cargo run -- story_demo.γλ` outputs *absolutely nothing*. There are no errors, no warnings, just complete silence!

## 🕵️ **The Reality:**
Turns out I used `τίτλος` as a variable when it's actually just a struct field, meaning the variable was undefined. The compiler simply ignores undefined variables in print statements and evaluates them to nothing without emitting any error, so `τίτλος λέγε.` just quietly outputs an empty string!

## 💡 **The Fix:**
The compiler MUST output an error when I try to use an undefined variable (e.g. "Undefined variable: τίτλος") instead of silently doing nothing. Also, add a huge banner in README saying 'REQUIRES FEATURE NOVA' or similar warnings to prevent DX pitfalls like this.
