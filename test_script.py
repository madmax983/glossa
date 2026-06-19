with open("src/codegen.rs", "r") as f:
    text = f.read()

# I see it printed `Stringifying main stmt 0` and then crashed! So `ms.to_string()` overflows the stack.
# `TokenStream::to_string()` relies on `Display` which is naturally recursive and doesn't use `stacker::maybe_grow`.

# The only way to fix it is either:
# 1. Leak the AST in the codegen test too. Since `TokenStream` and `quote!` are out of our control.
# Wait! In the journal it says:
# "When recursively formatting ASTs or generating code (e.g., in transpilers or narrators), avoid using `format!` as it causes O(n) intermediate `String` heap allocations. Instead, pass a `&mut String` buffer down the call stack and append directly using `std::fmt::Write` (`write!`) and `push_str()`."

# But `generate_rust` returns a `TokenStream` (from `proc-macro2`) internally, which IS a deep tree, and `to_string()` on `TokenStream` is out of our control!

# Is there any other memory? "To satisfy the Verification Rule..."
# "The Semantic AST types in Glossa ... To prevent stack overflows in tests that artificially construct deep ASTs, leak the AST (e.g., std::mem::forget)."
