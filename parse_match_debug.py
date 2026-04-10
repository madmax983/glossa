# The error happens in `compile_to_rust` inside `analyze_program`.
# `parse_match_expression` calls `skip_first_word_and_parse(&synthetic_clause, scope)`
# Wait! `skip_first_word_and_parse` does:
#  parse the rest of the clause, skipping the first word.
# In `κατὰ ξ`, the first word is `κατὰ`, and the rest is `ξ`.
# It uses `analyze_argument_expr` or `assemble_statement_allow_missing_verb`.
# Let's check `skip_first_word_and_parse`
