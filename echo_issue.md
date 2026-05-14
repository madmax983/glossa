# 🗣️ Echo: Getting Started example is broken / Missing variable errors fail silently

🤦 **The Confusion:**
I was following the "Troubleshooting" guide in `README.md` to learn about error messages. The README says that if I use a variable that hasn't been defined with `ἔστω`, I should get the error `Οὐκ οἶδα τὸ ὄνομα` ("I don't know the name"). So I wrote a simple script that just says an undefined variable:
```glossa
αγνωστον λέγε.
```
I ran it with `cargo run -- missing_variable.γλ`, but it didn't give me any error at all! The program just exited successfully and didn't print anything. It failed silently!

🕵️ **The Reality:**
It turns out that in the semantic analyzer, undefined variables silently evaluate to a default value like `Unknown` instead of explicitly returning an error. The compiler just swallows the problem and ignores the undefined variable when it tries to run the action.

💡 **The Fix:**
The compiler needs to be updated to actually return the `Οὐκ οἶδα τὸ ὄνομα` error when a variable isn't defined, just like the documentation says it should! Or, if it's supposed to fail silently, the README's Troubleshooting section needs to be fixed so it stops lying about this error message.
