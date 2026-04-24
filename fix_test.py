import re

with open('tests/havoc_repro.rs', 'r') as f:
    content = f.read()

# We need to make the variable defined, or we can just ignore this test as it was testing a bug.
# Let's change `λείτουργος` to a defined variable or just wrap it in a proper program. Wait, `λείτουργος ὁρίζειν` defines a function. Then `λείτουργος λέγε.` tries to call it.
# Actually, function calls are caught properly now. `λείτουργος` is defined as a function.
# Oh, `scope.is_defined(&"λειτουργος")` should return true for functions!
# But my code in `src/semantic/conversion.rs` checks `scope.is_defined(&subj.lemma)`.
