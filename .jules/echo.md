# 🗣️ Echo: Getting Started example is broken

**Scenario:** "I am a new user trying to run the 'Quick Start: The Hero's Journey' from `README.md` and check out the new `nova` map tool."

**Action:** I literally copy-pasted the code blocks from the README into `hero.γλ` and tried to run it. I also tried triggering an error on purpose and using the map tool as advertised.

### 🤦 The Confusion

1. **The Map Tool Example is Broken:** The README says "Generate a Mermaid.js class diagram of your code. `cargo run --release --features nova -- map examples/quickstart.γλ`". But when I ran that exact command, it printed `No architectural structures (Structs) found.`, even though there is clearly a `εἶδος Χρήστης` (User type) defined in that file!
2. **Helpful Errors Check Failed:** I intentionally tried to run a file that doesn't exist to see what error I'd get (`cargo run --release -- does_not_exist.γλ`). The message was `Error:   × Ἀρχεῖον οὐχ εὑρέθη: does_not_exist.γλ`. As an English speaker, I have NO idea what "Ἀρχεῖον οὐχ εὑρέθη" means. The troubleshooting table in the README has a few errors listed but not this one. How am I supposed to figure out "File not found"?
3. **The Import/Slang Check:** The instructions ask you to type `--features nova`. If I forget to type `--features nova`, the CLI gives me `error: unexpected argument 'examples/quickstart.γλ' found`. This is incredibly confusing—it doesn't tell me I'm missing a feature flag, it just says the filename is an unexpected argument!

### 🕵️ The Reality

1. The `map` tool is completely broken and doesn't detect structs properly, so the example in the README gives false output.
2. The compiler speaks entirely in Greek without an English fallback or an exhaustive translation table for basic things like "File not found" or "Is a directory".
3. The CLI parser throws a cryptic argument error when a feature flag is missing instead of a helpful error message.

### 💡 The Fix

1. Fix the `map` tool so it actually finds the `Χρήστης` struct from the Quick Start example, OR remove the `map` example from the README until it works.
2. Add a fallback or expand the troubleshooting table in the README so users can understand what "Ἀρχεῖον οὐχ εὑρέθη" means.
3. Add a better error message in the CLI telling the user "This command requires the 'nova' feature flag" instead of throwing an "unexpected argument" error.