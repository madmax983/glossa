# 🗣️ Echo: Confusing error when running experimental tools without the nova feature

🤦 **The Confusion:**
I'm trying out the new tools in ΓΛΩΣΣΑ from the "Quick Start: The Hero's Journey". I wanted to generate the architectural map of the `examples/quickstart.γλ` file.
So, I ran:
`cargo run --release -- map examples/quickstart.γλ`

The compiler immediately errored out with:
`error: unexpected argument 'examples/quickstart.γλ' found`

Wait, what? I'm passing the file directly to the map tool. Why is it saying the file itself is an unexpected argument?! Does `map` not take file arguments? I checked the README again, and it explicitly says to do this.

🕵️ **The Reality:**
It turns out that because `map` is an experimental tool guarded behind the `nova` feature, it simply *does not exist* in the CLI parser if I forget to pass `--features nova`.
Because the command `map` doesn't exist, clap thinks I'm trying to run the default action (`glossa [FILE]`), so it parses the word "map" as the `[FILE]` argument. Then, when it sees `examples/quickstart.γλ`, it throws an "unexpected argument" error because it already parsed the file name!

This is extremely confusing for users because the error message has absolutely nothing to do with missing a feature flag. I had to read the source code in `src/tools/cli.rs` and understand `#[cfg(feature = "nova")]` to figure this out.

💡 **The Fix:**
Do not compile away the experimental commands entirely!
Instead, the `map`, `mentor`, `mosaic`, etc. subcommands should *always* exist in the CLI parser. If a user tries to run them, but the compiler was compiled *without* the `nova` feature enabled, it should print a human-readable, friendly error:

> "The 'map' tool is an experimental feature. Please re-run the command with the `nova` feature enabled: `cargo run --features nova -- map <file>`"

This makes the tool much more idiot-proof.
