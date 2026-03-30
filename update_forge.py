with open(".jules/forge.md", "a") as f:
    f.write("\n\n**[Flattening God Functions in tester.rs and conjugation.rs]**\n")
    f.write("**Learning:** Large functions like `run_tests` and `analyze_verb` were getting unwieldy and doing too many things. In Rust, flattening logic into small, dedicated helpers and utilizing guard clauses dramatically improves clarity.\n")
    f.write("**Action:** Refactor God Functions into discrete helper methods and use early returns (guard clauses) to flatten Pyramids of Doom.\n")
