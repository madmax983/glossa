with open("src/tools/scholar.rs", "r") as f:
    for i, line in enumerate(f.readlines()):
        if 80 <= i + 1 <= 126:
            print(f"{i+1}: {line.rstrip()}")
