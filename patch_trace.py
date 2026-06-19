with open("src/codegen.rs", "r") as f:
    text = f.read()

# Instead of `output.to_string()`, let's try pushing to a string manually. Wait, `TokenStream` implements `Display`, but `Display` is recursive.
# If we do `output.to_string()`, it calls `Display::fmt` which is recursive.

# Wait, `generate_rust` creates a TokenStream, and then calls `to_string()`.
text = text.replace('    output.to_string()\n}', '''
    // TokenStream to string without deep recursion overflow
    // `output.to_string()` will overflow on deep ASTs.
    let mut s = String::new();
    for token in output {
        stacker::maybe_grow(32 * 1024, 2 * 1024 * 1024, || {
            s.push_str(&token.to_string());
            s.push(' ');
        });
    }
    s
}
''')

with open("src/codegen.rs", "w") as f:
    f.write(text)
