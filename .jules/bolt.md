**[Optimizing Collections in iterators]**
**Learning:** Checking for `.is_empty()` after `.collect::<Vec<_>>()` forces an unnecessary heap allocation. We can check if an iterator is empty dynamically using `.peekable()`. Additionally, if strings from `&'a str` need to be converted to `Cow<'static, str>`, we must use `.to_string()` as `.into()` or `Cow::Borrowed` will result in static lifetime compilation errors.
**Action:** Replace `Vec<_> = iter.collect()` with `.peekable()` for emptiness checking whenever possible, and be mindful of `Cow` lifetimes.
**[Replacing `push_str(&format!(...))` with `write!/writeln!`]
**Learning:** `push_str(&format!(...))` is an anti-pattern that creates a short-lived `String` allocation just to copy its contents into another `String`.
**Action:** Always use `write!(buf, ...)` or `writeln!(buf, ...)` from `std::fmt::Write` to format directly into an existing `String` buffer without intermediate allocations.

**[The `Vec::with_capacity` + `.extend()` vs `.collect()` Illusion]
**Learning:** Attempting to manually pre-allocate a `Vec` and calling `.extend()` over a simple `.map()` iterator is a placebo optimization. Rust's `.collect::<Vec<_>>()` already uses `Iterator::size_hint()` and internal traits (like `TrustedLen`) to perfectly pre-allocate the required capacity before inserting elements.
**Action:** Do not replace idiomatic `.collect()` calls with manual capacity pre-allocation when working with standard iterators, as it introduces verbosity with zero performance gain.
