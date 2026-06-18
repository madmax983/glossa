import re

with open("src/main.rs", "r") as f:
    content = f.read()

content = content.replace("Some(Commands::Gnomon { input }) => {", "Some(Commands::Gnomon { input }) => {\n            let _ = input;")
with open("src/main.rs", "w") as f:
    f.write(content)
