🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:**
1. I tried to follow the README's section "Running Code" which says to run `cargo run --release -- hero.γλ`. I got an error saying `Ἀρχεῖον οὐχ εὑρέθη: hero.γλ` (File not found).
2. The README's "Troubleshooting" section says that using an undefined variable results in "Οὐκ οἶδα τὸ ὄνομα" (I don't know the name). But when I run `ἄγνωστος λέγε.` (Say "unknown"), it just silently prints nothing and exits successfully! There is no error message at all.
3. The README says `cargo run --release -- map examples/quickstart.γλ` will run "The Cartographer", but the command fails unless I pass `--features nova`.

🕵️ **The Reality:**
1. The `hero.γλ` file doesn't actually exist in the repository. The user meant `examples/quickstart.γλ`.
2. The `λέγε` (print) function just silently ignores undefined variables instead of throwing the documented error. I dug deeper into the "Οὐκ οἶδα τὸ ὄνομα" error, but the actual error message emitted by the compiler for an undefined variable is `Τὸ «...» οὐχ ὡρίσθη — πρῶτον ὅρισον αὐτό` (The "..." is undefined - first define it). The documentation has the wrong error message *and* printing undefined variables doesn't even trigger it.
3. The README says `cargo run --release --features nova -- map examples/quickstart.γλ` is the correct command, but the earlier section uses the non-nova version `cargo run --release -- map examples/quickstart.γλ`.

💡 **The Fix:**
1. Update the README to replace `cargo run --release -- hero.γλ` with `cargo run --release -- examples/quickstart.γλ`.
2. Update the "Troubleshooting" table in the README: change "Οὐκ οἶδα τὸ ὄνομα" to "Τὸ «...» οὐχ ὡρίσθη — πρῶτον ὅρισον αὐτό" so it matches the actual compiler error. Also add a note that using `λέγε` with an undefined variable will currently just print nothing, as a known issue (until the compiler is fixed).
3. Fix the `nova` feature examples in the README to always include `--features nova`.