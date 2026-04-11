**Echo: Error messages in Troubleshooting guide don't exist**

**Learning:** Error messages in README like missing verb or undefined variable were silently swallowed by lenient assembler and conversion heuristics rather than explicitly triggering the defined `GlossaError`s and `AssemblyError`s.

**Action:** Tightened `Assembler::finalize` to catch verbless statements (returning `AssemblyError::MissingVerb`), and fortified heuristic fallback conversions (`try_print_default`, `classify_assignment`, etc) to return `GlossaError::undefined` when falling back to variable resolutions without an explicit type/scoping existing, while accommodating explicit exceptions like trait fields, `self`, and methods.
