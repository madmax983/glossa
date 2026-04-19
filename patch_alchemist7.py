with open("src/tools/alchemist.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if "let _program = AnalyzedProgram {" in line:
        new_lines.append(line.replace("let _program", "let mut _program"))
    elif "fn test_transpile_unimplemented_expr_fallback" in line:
        # replace the test with one that covers the fallback AND uses the program properly?
        # No, wait, if I put `let mut program` it was `unused_mut`. If I put `let program` it's `unused_variable`.
        # I just need to remove the program entirely since it's not used by `transpile_expr`.
        new_lines.append(line)
    else:
        new_lines.append(line)

with open("src/tools/alchemist.rs", "w") as f:
    f.writelines(new_lines)
