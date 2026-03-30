# 🗣️ Echo: Getting Started example is broken for `nova` feature commands

🤦 **The Confusion:**
I followed the instructions in `README.md` to run `cargo run --release -- map examples/quickstart.γλ` and `cargo run --release -- mentor`. The compiler output cryptically stated:
- For `map`: `error: unexpected argument 'examples/quickstart.γλ' found`
- For `mentor`: `Error:   × Ἀρχεῖον οὐχ εὑρέθη: mentor` (File not found)

🕵️ **The Reality:**
The `README.md` instructions correctly mentioned that these advanced tools require the `nova` feature (i.e. `cargo run --release --features nova -- map ...`). However, if a user happens to miss this flag, the CLI framework fails cryptically instead of telling them what went wrong. It either treats the command name (`mentor`) as a target script to run or completely misinterprets the input.

💡 **The Fix:**
The CLI should unconditionally accept these commands in `clap`, but run a block that prints a helpful error message if the `nova` feature is not enabled. For example: "Error: The `mentor` command requires the `nova` feature. Re-run with `--features nova`." This prevents the user from going on a wild goose chase.
