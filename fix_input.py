import re

with open("src/main.rs", "r") as f:
    content = f.read()

content = content.replace("Some(Commands::Gnomon { input }) => {", "Some(Commands::Gnomon { input: _ }) => {")
with open("src/main.rs", "w") as f:
    f.write(content)
