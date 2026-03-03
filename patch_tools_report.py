with open('src/tools/report.rs', 'r') as f:
    lines = f.readlines()

out = []
for line in lines:
    if "trait_count: program.scope.traits().count()" in line:
        continue
    out.append(line)

with open('src/tools/report.rs', 'w') as f:
    f.writelines(out)
