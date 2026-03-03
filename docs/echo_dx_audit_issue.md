# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
I was reading the README to figure out how to write a simple loop. I saw this in the "Rosetta Stone" table:

`διὰ λίστης, ξ λέγε.` -> `for x in list { ... }`

So I copy-pasted that into my script and hit run:

```
> cargo run --release -- my_script.γλ

Σφάλμα κώδικος (Codegen Error)
Error:   ×
  │ ╔══════════════════════════════════════════════════════════════╗
  │ ║  INTERNAL COMPILER ERROR (Codegen Failed)                    ║
  │ ╚══════════════════════════════════════════════════════════════╝
  │
  │ This indicates a bug in the Glossa compiler's code generation.
  │ Please report this issue with the following details:
  │
  │ error[E0425]: cannot find value `g__u3bb__u3b9__u3c3__u3c4__u3b7__u3c2_`
  │ in this scope
```

**🕵️ The Reality:**
1. The code in the example assumes `λίστης` (or some array/list) was already defined. Because it wasn't, instead of getting a helpful "Hey, I don't know what `λίστης` is" (like the Troubleshooting table claims with "Οὐκ οἶδα τὸ ὄνομα"), it gives me a gigantic, scary Rust "INTERNAL COMPILER ERROR".
2. Even if I *do* define the list first (e.g., `λίστα [1, 2, 3] ἔστω.`), if I then try `διὰ λίστης, ξ λέγε.`, it still throws the exact same raw `rustc` error! The compiler seems to fail to match the Genitive (`λίστης`) to the Nominative (`λίστα`), leaking the Rust codegen failure to the user.

**💡 The Fix:**
1. Fix the error handling! If I use a variable that hasn't been defined in a loop, it should say "Οὐκ οἶδα τὸ ὄνομα", not throw an internal compiler error full of random characters.
2. In the README example table, change the loop example to one that actually works if copy-pasted, or specify that a list must be defined first. It seems `διὰ ἀριθμοί, ν λέγε.` works perfectly in the `quickstart.γλ` file, so let's update the README to use that example.

---
*Echo's philosophy:*
*If I copy-paste the example and it doesn't compile, I am leaving.*
*Error messages should be helpful.*