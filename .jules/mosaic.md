
**[Consistent Dashboard Formatting]
**Learning:** `comfy-table`'s default `Display` implementation (`println!("{table}")`) does not respect surrounding margin or indent contexts.
**Action:** When printing tables within a styled CLI dashboard, iterate over the rendered string lines (`for line in table.to_string().lines()`) and manually apply the standard indentation prefix (`println!("   {}", line)`) to maintain visual alignment.
