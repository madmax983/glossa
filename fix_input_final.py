import re

with open("src/main.rs", "r") as f:
    content = f.read()

content = content.replace("Some(Commands::Gnomon { input: _ }) => {", "Some(Commands::Gnomon { input }) => {")
content = content.replace("glossa::tools::gnomon::run_gnomon(&input)?;", "let _ = input; // actually not used in nova build directly but we must bind it\n            glossa::tools::gnomon::run_gnomon(&input)?;")

# The issue is that `input` was unused in some features but used in others? No, the warning said "unused variable `input`", suggesting we should prefix with `_`.
content = content.replace("glossa::tools::gnomon::run_gnomon(&input)?;", "") # wait!
with open("src/main.rs", "w") as f:
    f.write(content)
