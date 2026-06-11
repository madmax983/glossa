content = """**Refactored Report tool visit_statement logic**
**Learning:** Found a Pyramid of Doom with deep nesting inside `TraitImplementation` match arm.
**Action:** Extracted the logic into a separate `visit_methods_body` helper function to flatten the structure using an early return.
"""

with open('.jules/forge.md', 'a') as f:
    f.write(content)
