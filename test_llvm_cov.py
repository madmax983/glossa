import sys
import re

with open("target/llvm-cov/html/coverage/app/src/tools/diplomat.rs.html", "r") as f:
    content = f.read()

lines = content.split("<tr>")
for line in lines:
    if "uncovered-line" in line:
        print("UNCOVERED:", line)
