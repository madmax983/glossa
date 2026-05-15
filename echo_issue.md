# 🗣️ Echo: "File not found" errors are in Ancient Greek

🤦 **The Confusion:**
When I mistyped a file name in the CLI (like `cargo run -- oops.γλ`), I got this error:
`Error:   × Ἀρχεῖον οὐχ εὑρέθη: oops.γλ`

I literally had to open Google Translate. I get the whole "Ancient Greek" gimmick for syntax errors, but for a simple "File Not Found" system error, this is terrible. If I can't even get the compiler to run, I shouldn't need a dictionary.

🕵️ **The Reality:**
The basic file validation outputs `Ἀρχεῖον οὐχ εὑρέθη` instead of simple English. I am the dumbest person in the room, and if I can't read the very first error I get before my code even compiles, I am going to quit and use Python instead.

💡 **The Fix:**
Change the basic OS-level errors to be English first, or at least provide the English translation alongside it. For example: `File not found (Ἀρχεῖον οὐχ εὑρέθη): oops.γλ`. Users should not need a Rosetta Stone to figure out they mistyped the filename!
