**Extracted execute_command**
**Learning:** The CLI command routing logic in src/main.rs was heavily bloated by a massive match statement dealing with feature flags and different tool options, which obscured the flow of the main() function.
**Action:** Extracted the routing block into a dedicated execute_command helper function, preserving the original behavior while drastically flattening main().
