import re

with open("src/codegen.rs", "r") as f:
    text = f.read()

text = text.replace('pub fn generate_rust(program: &AnalyzedProgram) -> String {', 'pub fn generate_rust(program: &AnalyzedProgram) -> String { println!("Entering generate_rust");')

with open("src/codegen.rs", "w") as f:
    f.write(text)
