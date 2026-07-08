## JSON Output Removal
**Before:** The `papyrus` tool printed raw `JSONB` strings straight into the terminal when converting generic list/map types. And `tester` printed raw error stack traces when tests failed.
**After:** We refactored `papyrus` to construct properly formatted `String::from("JSONB")` outputs and updated `tester.rs` to place unparseable raw test output cleanly inside `comfy_table` styled blocks, eliminating direct raw dump to the terminal. We also updated empty test runs to inform the user nicely.
**Visuals:** Clean tabular error layouts and valid string conversions.
