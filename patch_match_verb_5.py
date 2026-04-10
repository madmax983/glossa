# The error happens in `compile_to_rust(source)` which does `analyze_program(&ast)`.
# The program has two statements:
# 1. `ξ πέντε ἔστω.`
# 2. `κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἓν ᾖ, «ἕν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.`
# The second statement is passed to `analyze_statement`, which first checks control flow:
# `analyze_control_flow(stmt, scope)` -> `parse_match(stmt, scope)`
# Wait! Does `parse_match` successfully parse it?
# Let's check `parse_match` in `src/semantic/control_flow.rs`.
