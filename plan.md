1. **Explore Codebase for "Enterprise FizzBuzz" patterns**
   - Use `grep` to identify structural abstractions like `*Visitor`, `*Builder`, `*Generator` across the codebase. We already flattened `AuditorVisitor` and `GnomonVisitor` into procedural functions as per Razor's philosophy.
2. **Review Journal (`.jules/razor.md`)**
   - I have read and reviewed the user's specific guidelines.
3. **Clean Up temporary python files**
   - Run a clean up command `rm replace_*.py fix_*.py`
4. **Log critical learnings to `.jules/razor.md`**
   - Record the reduction of `AuditorVisitor` and `GnomonVisitor` into standard procedural Rust functions.
5. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
6. **Submit PR**
   - With the commit title starting with "🪒 Razor: "
