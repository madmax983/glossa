with open('src/errors/mod.rs', 'r') as f:
    lines = f.readlines()

# find test module start
test_start = 0
for i, line in enumerate(lines):
    if "mod tests {" in line:
        test_start = i - 1 # account for #[cfg(test)]
        break

# remove the pub use at the end
to_move = []
for i in range(len(lines)-1, -1, -1):
    if "pub use help::" in lines[i]:
        to_move.append(lines.pop(i))

to_move.reverse()

lines.insert(test_start, "".join(to_move) + "\n")

with open('src/errors/mod.rs', 'w') as f:
    f.writelines(lines)
