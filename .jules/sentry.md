
**[Coverage Gap: Trivial Enum Variant `Drop`]
**Learning:** `std::mem::replace` is a powerful trick to bypass Rust's drop checker bypassing early struct instantiation drops and forcing a variable to hold a different internal state to trigger unhandled `_ => {}` match arms during natural scoping end.
**Action:** Use `std::mem::replace` to swap internal states just before dropping when trying to hit fallback variants of complex enum `Drop` implementations.
