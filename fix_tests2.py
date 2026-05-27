with open('src/tools/sculptor.rs', 'r') as f:
    content = f.read()

# We messed up the replacement. Let's just find the closing } of `mod tests` and move it to the end of the file.
lines = content.split('\n')
for i, line in enumerate(lines):
    if line == "    }":
        pass

# A simpler way:
# Restore the file to original state and do it manually
