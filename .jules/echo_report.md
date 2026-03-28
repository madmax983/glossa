## 🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:**
Tried to run the `hero.γλ` example command from the "Running Code" section of the README (`cargo run --release -- hero.γλ`). The compiler returned an error: `Error: × Ἀρχεῖον οὐχ εὑρέθη: examples/hero.γλ` (File not found).

🕵️ **The Reality:**
Turns out `hero.γλ` doesn't actually exist in the repository. It was just meant as a placeholder name for "any ΓΛΩΣΣΑ file", but as a new user, I expect to be able to copy-paste the exact commands provided and have them work immediately.

💡 **The Fix:**
Change the documentation to use a real, runnable file from the repository, such as `examples/quickstart.γλ`, or provide instructions to create `hero.γλ` first.
