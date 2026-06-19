with open("tests/havoc_codegen_stack_overflow.rs", "r") as f:
    text = f.read()

# Add thread spawn to havoc_codegen_stack_overflow!
# Actually, the problem is proc_macro2 limits stringification. I need to make generate_rust NOT overflow.
