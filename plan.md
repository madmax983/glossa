Oh, I see. `inline_codegen.py` deleted the functions, and I ran `cargo test`, and there was an error.
Then I ran `git restore src/codegen.rs`, but apparently it didn't restore completely because I already committed it? No, wait! The python script I just ran did:
```python
with open('src/codegen.rs', 'w') as f:
    f.write(content)
```
And it DID remove the functions. I see `fn generate_expr(` is on line 913 now instead of 928, meaning the functions ABOVE it were removed successfully!
But why did it fail with `unexpected closing delimiter` on line 903?
Because `generate_test` ends with a brace, and my regex `r'fn generate_expr_some.*?quote! \{ Some\(#inner_tokens\) \}\n\}'` matched `generate_test`'s end brace as well maybe? Wait, no.

Let me restore the file from HEAD.
