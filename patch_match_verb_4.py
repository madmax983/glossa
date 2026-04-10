with open("src/semantic/conversion.rs", "r") as f:
    content = f.read()

# Wait... if `exprs` is empty and `subject` is None and `object` is None, then `classify_expression` returns `Ok(AnalyzedStatement::Expression(vec![]))`
# And then `extract_value` calls `classify_expression`?
# NO, `extract_value` does NOT call `classify_expression`!
# Let's check `parse_match_pattern` in `src/semantic/control_flow.rs`.
