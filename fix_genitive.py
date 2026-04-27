import re

with open("src/semantic/conversion.rs", "r") as f:
    content = f.read()

# I will skip the frontend instructions as there is no frontend here.
