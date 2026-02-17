# Bolt's Journal ⚡

## [Memory] Zero-Allocation Prefix Matching
**Learning:** Matching strings against a set of prefixes/suffixes is often implemented with `Vec::new()`, `collect()`, and `sort()`. This allocates heap memory on every call. If the patterns are static, pre-sort them in the source code (by length descending for longest-match) and iterate the static slice directly. This turns O(N log N) + Allocation into O(N) stack-only scan.
**Action:** Always check if constant data can be pre-sorted to avoid runtime sorting. Add a regression test to enforce the sort order invariant.

## [Performance] Optimistic vs Pessimistic Checks
**Learning:** Adding a "fast path" check (e.g., iterating to detect if normalization is needed) can cause regressions if the "fast path" fails frequently (dirty data). The double iteration cost outweighs the saved allocation.
**Action:** Profile the data distribution. If data is mostly dirty, optimize the dirty path (e.g. `String::with_capacity`) rather than adding optimistic checks that require re-scanning.

## [Performance] Complex Unicode Casing vs Allocation
**Learning:** `char::to_lowercase` is efficient (iterator) but incorrect for context-sensitive casing (like Greek final sigma). `String::to_lowercase` is correct but allocates. A hybrid approach checking for the presence of uppercase characters allows using the fast path for the common case (lowercase identifiers) while preserving correctness for edge cases.
**Action:** When optimizing casing operations, check if the input is already in a state that allows a simpler, allocation-free transformation.
**[Normalization Caching Strategy]**
**Learning:** When caching a normalized form (e.g. ) in a struct to avoid re-computation, ensure that:
1. The API to populate the struct takes the pre-computed normalized form or computes it once at the boundary.
2. If computing it at the boundary, update all internal helper functions to propagate it, avoiding shadowing or re-computation.
3. Update all test helpers or manual struct instantiations to include the new field.
4. Watch out for  when converting  to  via .
**Action:** When adding a cache field, grep for all instantiations of the struct and all calls to the constructor/feed functions.
**[Coverage vs Compilation]**
**Learning:** A failing CI coverage check might be a symptom of a build failure in the test suite itself, especially if new tests were added but not compiling. Codecov often reports low coverage if the tests exercising the new code failed to run.
**Action:** Always check the build logs for the test suite before assuming the coverage report means "tests ran but missed lines". Ensure
running 306 tests
test codegen::tests::test_basic_types ... ok
test codegen::tests::test_capitalize ... ok
test codegen::tests::test_container_types ... ok
test codegen::tests::test_generate_number ... ok
test codegen::tests::test_generate_hello ... ok
test codegen::tests::test_generate_binding ... ok
test codegen::tests::test_generate_full_program ... ok
test codegen::tests::test_generate_unreachable_operators ... ok
test codegen::tests::test_sanitize_greek_letter ... ok
test codegen::tests::test_sanitize_keywords_and_prefix ... ok
test codegen::tests::test_generate_statement_code ... ok
test codegen::tests::test_struct_type ... ok
test codegen::tests::test_transliterate_mixed_valid_invalid ... ok
test codegen::tests::test_transliterate ... ok
test errors::messages::tests::test_gender_mismatch_message ... ok
test errors::messages::tests::test_undefined_variable_message ... ok
test codegen::tests::test_transliterate_unique ... ok
test errors::tests::test_category_greek ... ok
test errors::tests::test_parse_error ... ok
test errors::tests::test_undefined_error ... ok
test grammar::tests::test_parse_ascii_question_mark ... ok
test grammar::tests::test_parse_chained_statements ... ok
test grammar::tests::test_parse_genitive_property_access ... ok
test grammar::tests::test_parse_hello_cosmos ... ok
test grammar::tests::test_parse_greek_question_mark ... ok
test grammar::tests::test_parse_inline_comment ... ok
test grammar::tests::test_parse_line_comment ... ok
test grammar::tests::test_parse_simple_string_print ... ok
test grammar::tests::test_parse_variable_binding ... ok
test grammar::tests::test_parse_variable_use ... ok
test morphology::conjugation::tests::test_analyze_verb_all_discrepancies ... ok
test grammar::tests::test_parse_number_literal ... ok
test morphology::conjugation::tests::test_aorist_active ... ok
test grammar::tests::test_parse_multiple_statements ... ok
test morphology::conjugation::tests::test_analyze_verb_coverage_forms ... ok
test morphology::conjugation::tests::test_augment_stripping ... ok
test morphology::conjugation::tests::test_conjugate ... ok
test morphology::conjugation::tests::test_eimi_subjunctive ... ok
test morphology::conjugation::tests::test_constants_sorted ... ok
test morphology::conjugation::tests::test_infinitive ... ok
test morphology::conjugation::tests::test_infinitive_form ... ok
test morphology::conjugation::tests::test_present_active_imperative ... ok
test morphology::conjugation::tests::test_present_active_indicative ... ok
test morphology::conjugation::tests::test_strip_augment_function ... ok
test morphology::declension::tests::test_analyze_noun_all_ambiguity ... ok
test morphology::declension::tests::test_analyze_noun_disambiguation_alpha ... ok
test morphology::declension::tests::test_constants_sorted ... ok
test morphology::declension::tests::test_decline_alpha_mismatch ... ok
test morphology::declension::tests::test_decline_second_masculine ... ok
test morphology::declension::tests::test_decline_fallthrough ... ok
test morphology::declension::tests::test_decline_table ... ok
test morphology::declension::tests::test_first_declension_eta ... ok
test morphology::declension::tests::test_second_declension_accusative ... ok
test morphology::declension::tests::test_get_stem ... ok
test morphology::declension::tests::test_second_declension_dative ... ok
test morphology::declension::tests::test_first_declension_alpha ... ok
test morphology::declension::tests::test_second_declension_genitive ... ok
test morphology::declension::tests::test_second_declension_neuter_plural ... ok
test morphology::declension::tests::test_second_declension_nominative ... ok
test morphology::declension::tests::test_third_declension_ma ... ok
test morphology::disambiguation::tests::test_article_context ... ok
test morphology::declension::tests::test_second_declension_vocative ... ok
test morphology::declension::tests::test_second_declension_plural ... ok
test morphology::disambiguation::tests::test_disambiguate_with_article ... ok
test morphology::disambiguation::tests::test_neuter_article_ambiguity ... ok
test morphology::disambiguation::tests::test_no_context_preserves_order ... ok
test morphology::disambiguation::tests::test_verb_context ... ok
test morphology::lexicon::tests::test_arithmetic_operators ... ok
test morphology::lexicon::tests::test_assert_verb_lexicon_entry ... ok
test morphology::lexicon::tests::test_boolean_lookup ... ok
test morphology::lexicon::tests::test_declined_collection_nouns ... ok
test morphology::lexicon::tests::test_boolean_operators ... ok
test morphology::lexicon::tests::test_equals_verb_lexicon_entry ... ok
test morphology::lexicon::tests::test_comparison_operators ... ok
test morphology::lexicon::tests::test_gignetai_is_assignment_verb ... ok
test morphology::lexicon::tests::test_gignetai_lexicon_entry ... ok
test morphology::lexicon::tests::test_insert_verb_lexicon_entries ... ok
test morphology::lexicon::tests::test_is_assert_verb ... ok
test morphology::lexicon::tests::test_is_containment_preposition ... ok
test morphology::lexicon::tests::test_is_delimiter_preposition ... ok
test morphology::lexicon::tests::test_is_insert_verb ... ok
test morphology::lexicon::tests::test_is_join_verb ... ok
test morphology::lexicon::tests::test_is_print_verb ... ok
test morphology::lexicon::tests::test_is_split_verb ... ok
test morphology::lexicon::tests::test_join_verb_lexicon_entries ... ok
test morphology::lexicon::tests::test_lookup_binding ... ok
test morphology::lexicon::tests::test_lookup_type ... ok
test morphology::lexicon::tests::test_lookup_verb ... ok
test morphology::lexicon::tests::test_meta_lexicon_entry ... ok
test morphology::lexicon::tests::test_meta_is_mutable_marker ... ok
test morphology::lexicon::tests::test_non_assignment_verb ... ok
test morphology::lexicon::tests::test_non_mutable_marker ... ok
test morphology::lexicon::tests::test_numeral_value ... ok
test morphology::lexicon::tests::test_operator_lexicon_entries ... ok
test morphology::lexicon::tests::test_split_verb_lexicon_entries ... ok
test morphology::participle::tests::test_aorist_active_participle ... ok
test morphology::participle::tests::test_aorist_feminine ... ok
test morphology::participle::tests::test_aorist_neuter ... ok
test morphology::participle::tests::test_perfect_passive_participle ... ok
test morphology::participle::tests::test_present_active_participle_feminine ... ok
test morphology::participle::tests::test_present_active_participle_masculine ... ok
test morphology::participle::tests::test_present_active_participle_neuter ... ok
test morphology::participle::tests::test_present_middle_participle ... ok
test morphology::participle::tests::test_verb_lemma ... ok
test morphology::tests::test_ambiguous_word_analysis ... ok
test morphology::tests::test_analyze_accusative ... ok
test morphology::tests::test_analyze_all_coverage_forms ... ok
test morphology::tests::test_analyze_dative ... ok
test morphology::tests::test_analyze_genitive ... ok
test morphology::tests::test_analyze_lexicon_lookup ... ok
test morphology::lexicon::tests::test_is_equals_verb ... ok
test morphology::tests::test_analyze_verb_imperative ... ok
test morphology::tests::test_analyze_nominative ... ok
test morphology::tests::test_analyze_verb_present ... ok
test morphology::tests::test_display_impls ... ok
test morphology::tests::test_sort_safety_with_nan ... ok
test morphology::tests::test_single_greek_letter_fallback ... ok
test morphology::tests::test_unknown_word_fallback ... ok
test parser::numerals::tests::test_2024 ... ok
test parser::numerals::tests::test_archaic ... ok
test parser::numerals::tests::test_hundreds ... ok
test parser::numerals::tests::test_invalid ... ok
test parser::numerals::tests::test_sigma_uppercase ... ok
test parser::numerals::tests::test_teens ... ok
test parser::numerals::tests::test_tens ... ok
test parser::numerals::tests::test_units ... ok
test parser::numerals::tests::test_thousands ... ok
test parser::numerals::tests::test_full_coverage ... ok
test parser::tests::test_parse_source_hello ... ok
test parser::tests::test_parse_source_number_literal ... ok
test parser::tests::test_parse_source_query ... ok
test parser::tests::test_parse_source_multiple_statements ... ok
test parser::tests::test_parse_source_string_literal ... ok
test parser::tests::test_recursion_limit_exceeded ... ok
test parser::tests::test_recursion_limit_ignored_in_comment ... ok
test parser::tests::test_parse_source_variable_binding ... ok
test parser::tests::test_recursion_limit_mixed_brackets ... ok
test parser::tests::test_parse_source_with_comma ... ok
test parser::tests::test_recursion_limit_ignored_in_string ... ok
test parser::tests::test_recursion_limit_not_exceeded ... ok
test parser::tests::test_recursion_limit_unbalanced_but_safe ... ok
test report::tests::test_program_stats_coverage ... ok
test report::tests::test_report_manual_ast_coverage ... ok
test parser::tests::test_word_normalization ... ok
test report::tests::test_compilation_report_coverage ... ok
test semantic::assembler::tests::test_boolean_or_detection ... ok
test semantic::assembler::tests::test_dative_indirect_object ... ok
test report::tests::test_report_generation_coverage ... ok
test semantic::assembler::tests::test_disambiguation_en_vs_hen ... ok
test semantic::assembler::tests::test_double_object_error ... ok
test semantic::assembler::tests::test_double_verb_error ... ok
test semantic::assembler::tests::test_full_boolean_or_expression ... ok
test semantic::assembler::tests::test_gender_mismatch_ignored ... ok
test semantic::assembler::tests::test_immediate_agreement_failure_svo ... ok
test semantic::assembler::tests::test_immediate_agreement_failure_vso ... ok
test semantic::assembler::tests::test_genitive_possession ... ok
test semantic::assembler::tests::test_imperative_mismatch ... ok
test semantic::assembler::tests::test_literals ... ok
test semantic::assembler::tests::test_max_literals_exceeded ... ok
test semantic::assembler::tests::test_max_genitives_exceeded ... ok
test semantic::assembler::tests::test_max_nominatives_exceeded ... ok
test semantic::assembler::tests::test_max_operators_exceeded ... ok
test semantic::assembler::tests::test_neuter_plural_subject_first_person_verb ... ok
test semantic::assembler::tests::test_multiple_nominatives ... ok
test semantic::assembler::tests::test_neuter_plural_subject_singular_verb ... ok
test semantic::assembler::tests::test_operator_detection ... ok
test semantic::assembler::tests::test_silent_swallowing_of_unknown_case ... ok
test semantic::assembler::tests::test_simple_sov ... ok
test semantic::assembler::tests::test_split_method_generation ... ok
test semantic::assembler::tests::test_subject_verb_person_agreement ... ok
test semantic::assembler::tests::test_verb_constituent_has_voice ... ok
test semantic::assembler::tests::test_vso_same_result ... ok
test semantic::expressions::tests::test_analyze_argument_expr_errors_on_block_with_empty_clause ... ok
test semantic::expressions::tests::test_analyze_argument_expr_errors_on_empty_block ... ok
test semantic::assembler::tests::test_max_adjectives_exceeded ... ok
test semantic::expressions::tests::test_analyze_argument_expr_errors_on_invalid_property ... ok
test semantic::expressions::tests::test_analyze_argument_expr_errors_on_invalid_block_structure ... ok
test semantic::expressions::tests::test_analyze_argument_expr_errors_on_empty_phrase ... ok
test semantic::expressions::tests::test_analyze_argument_expr_handles_array ... ok
test semantic::expressions::tests::test_analyze_argument_expr_handles_binop ... ok
test semantic::expressions::tests::test_analyze_argument_expr_handles_block ... ok
test semantic::expressions::tests::test_analyze_argument_expr_handles_function_call ... ok
test semantic::expressions::tests::test_analyze_argument_expr_handles_unwrap ... ok
test semantic::expressions::tests::test_analyze_argument_expr_handles_phrase_recursion ... ok
test semantic::expressions::tests::test_analyze_argument_expr_handles_index_access ... ok
test semantic::expressions::tests::test_analyze_argument_expr_handles_property_access ... ok
test semantic::expressions::tests::test_analyze_argument_expr_propagates_error_in_array ... ok
test semantic::expressions::tests::test_analyze_argument_expr_propagates_error_in_binop ... ok
test semantic::expressions::tests::test_analyze_argument_expr_propagates_error_in_function_args ... ok
test semantic::expressions::tests::test_analyze_argument_expr_propagates_error_in_property_owner ... ok
test semantic::expressions::tests::test_analyze_argument_expr_propagates_error_in_unary_op ... ok
test semantic::expressions::tests::test_build_expressions_preserves_literals ... ok
test semantic::expressions::tests::test_backtracking_failure_propagates_error ... ok
test semantic::expressions::tests::test_analyze_argument_expr_propagates_error_in_index_access ... ok
test semantic::expressions::tests::test_phrase_errors_on_multiple_terms ... ok
test semantic::expressions::tests::test_recursion_limit_expression_analysis ... ok
test semantic::expressions::tests::test_vso_ambiguity_resolution ... ok
test semantic::resolver::tests::test_child_scope_inherits ... ok
test semantic::resolver::tests::test_child_scope_shadows ... ok
test semantic::resolver::tests::test_enter_exit_scope ... ok
test semantic::resolver::tests::test_enter_shadowing ... ok
test semantic::resolver::tests::test_lookup_binding_immutable ... ok
test semantic::resolver::tests::test_lookup_binding_not_found ... ok
test semantic::resolver::tests::test_lookup_binding_parent_scope ... ok
test semantic::resolver::tests::test_lookup_binding_returns_full_binding ... ok
test semantic::resolver::tests::test_mark_used ... ok
test semantic::resolver::tests::test_mutable_binding ... ok
test semantic::resolver::tests::test_scope_function_coverage ... ok
test semantic::resolver::tests::test_scope_trait_coverage ... ok
test semantic::resolver::tests::test_scope_type_coverage ... ok
test semantic::resolver::tests::test_shadowing_across_types ... ok
test semantic::resolver::tests::test_scope_define_and_lookup ... ok
test semantic::resolver::tests::test_scope_mixed_namespace_collisions ... ok
test semantic::resolver::tests::test_unused_bindings ... ok
test semantic::tests::test_analyze_number_literal ... ok
test semantic::tests::test_analyze_hello ... ok
test semantic::tests::test_analyze_binding ... ok
test semantic::types::tests::test_display_formatting ... ok
test semantic::types::tests::test_detect_collection_type ... ok
test semantic::tests::test_analyze_string_literal ... ok
test semantic::tests::test_analyze_variable_use ... ok
test semantic::types::tests::test_type_compatibility ... ok
test semantic::types::tests::test_type_to_greek ... ok
test text::tests::test_normalize_anthropos ... ok
test text::tests::test_normalize_athenai ... ok
test text::tests::test_normalize_case_insensitive ... ok
test text::tests::test_normalize_chaire ... ok
test text::tests::test_normalize_circumflex ... ok
test text::tests::test_normalize_genitive_ending ... ok
test text::tests::test_normalize_esto ... ok
test text::tests::test_normalize_iota_subscript ... ok
test text::tests::test_normalize_lege ... ok
test text::tests::test_normalize_meizon ... ok
test text::tests::test_normalize_mixed_text ... ok
test text::tests::test_normalize_or_particle ... ok
test text::tests::test_normalize_preserves_base_letters ... ok
test text::tests::test_normalize_rough_breathing ... ok
test text::tests::test_normalize_subjunctive_eimi ... ok
test tools::dictionary::tests::test_lookup_known_word ... ok
test tools::dictionary::tests::test_lookup_unknown_word ... ok
test tools::highlight::tests::test_highlight_all_binops ... ok
test tools::dictionary::tests::test_lookup_verb ... ok
test tools::highlight::tests::test_highlight_array_literal ... ok
test tools::highlight::tests::test_highlight_article_context ... ok
test tools::highlight::tests::test_highlight_binop ... ok
test tools::highlight::tests::test_highlight_block ... ok
test tools::highlight::tests::test_highlight_binding ... ok
test tools::highlight::tests::test_highlight_boolean_literal ... ok
test tools::highlight::tests::test_highlight_clause_separator ... ok
test tools::highlight::tests::test_highlight_complex_nested ... ok
test tools::highlight::tests::test_highlight_dative ... ok
test tools::highlight::tests::test_highlight_definitions ... ok
test tools::highlight::tests::test_highlight_error ... ok
test tools::highlight::tests::test_highlight_definitions_formatting ... ok
test tools::highlight::tests::test_highlight_index_access ... ok
test tools::highlight::tests::test_highlight_function_call ... ok
test tools::highlight::tests::test_highlight_multiple_statements ... ok
test tools::highlight::tests::test_highlight_nested_phrase ... ok
test tools::highlight::tests::test_highlight_number_literal ... ok
test tools::highlight::tests::test_highlight_participle ... ok
test tools::highlight::tests::test_highlight_phrase ... ok
test tools::highlight::tests::test_highlight_propagate ... ok
test tools::highlight::tests::test_highlight_pos_variants ... ok
test tools::highlight::tests::test_highlight_property_access ... ok
test tools::highlight::tests::test_highlight_query ... ok
test tools::highlight::tests::test_highlight_string_literal ... ok
test tools::highlight::tests::test_highlight_simple_sentence ... ok
test tools::highlight::tests::test_highlight_test_declaration ... ok
test tools::highlight::tests::test_highlight_unary_neg ... ok
test tools::highlight::tests::test_highlight_trait_impl ... ok
test tools::highlight::tests::test_highlight_unary_op ... ok
test tools::highlight::tests::test_manual_ast_nodes ... ok
test tools::highlight::tests::test_highlight_vocative_and_adjective ... ok
test tools::repl::tests::test_print_banner_coverage ... ok
test tools::narrator::tests::test_bard_basic ... ok
test tools::narrator::tests::test_bard_print ... ok
test tools::repl::tests::test_print_env_empty ... ok
test tools::repl::tests::test_print_help_coverage ... ok
test tools::repl::tests::test_repl_execute_variants ... ok
test tools::repl::tests::test_repl_env_coverage ... ok
test tools::repl::tests::test_repl_output_display ... ok
test tools::repl::tests::test_run_repl_inner_eof ... ok
test tools::repl::tests::test_run_repl_inner_help ... ok
test tools::repl::tests::test_run_repl_inner_workflow ... ok
test tools::runner::tests::test_bard_file_valid ... ok
test tools::repl::tests::test_repl_source_limit ... ok
test tools::runner::tests::test_build_file_success ... ok
test tools::runner::tests::test_build_file_size_limit ... ok
test tools::runner::tests::test_compile_full_program ... ok
test tools::runner::tests::test_check_file_valid ... ok
test tools::runner::tests::test_compile_binding ... ok
test tools::runner::tests::test_file_size_check_internal ... ok
test tools::runner::tests::test_highlight_file_valid ... ok
test tools::runner::tests::test_compile_hello ... ok
test tools::runner::tests::test_run_compile_error ... ok
test tools::runner::tests::test_run_file_size_limit ... ok
test tools::ui::tests::test_status_drop ... ok
test tools::ui::tests::test_status_no_tty_error ... ok
test tools::ui::tests::test_status_no_tty_success ... ok
test tools::ui::tests::test_status_no_tty_update ... ok
test tools::ui::tests::test_status_tty_error ... ok
test tools::ui::tests::test_status_tty_success ... ok
test tools::ui::tests::test_status_tty_update ... ok
test tools::runner::tests::test_run_rustc_error ... ok
test tools::repl::tests::test_repl_binding_limit ... ok
test
test
test tools::runner::tests::test_run_file_success ... ok

test result: ok. 306 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 12 tests
test test_assembled_statement_derive_coverage ... ok
test test_assembler_has_content_coverage ... ok
test test_assembler_boolean_and_coverage ... ok
test test_assembler_arithmetic_operators_coverage ... ok
test test_assembler_error_cases_coverage ... ok
test test_assembler_method_verbs_join_coverage ... ok
test test_assembler_method_verbs_split_coverage ... ok
test test_assembler_numeral_coverage ... ok
test test_assembler_set_flags_coverage ... ok
test test_assembler_ordinal_index_coverage ... ok
test test_assembler_special_markers_coverage ... ok
test test_constituent_derive_coverage ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
test test_comparison_fallback ... ok
test test_boolean_ops_fallback ... ok
test test_number_boolean_ops_codegen ... ok
test test_comparison_ops_fallback ... ok
test test_arithmetic_ops_fallback ... ok
test test_string_concatenation_fallback ... ok
test test_number_comparison_ops_codegen ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 27 tests
test test_codegen_empty_array ... ok
test test_codegen_array_literal ... ok
test test_array_iteration ... ok
test test_array_iteration_with_body ... ok
test test_hashmap_contains_key ... ok
test test_hashmap_get ... ok
test test_hashset_contains ... ok
test test_hashmap_insert ... ok
test test_hashset_insert ... ok
test test_hashset_insert_string ... ok
test test_length_property ... ok
test test_length_multiple ... ok
test test_numeric_index ... ok
test test_numeric_index_expression ... ok
test test_ordinal_index_first ... ok
test test_ordinal_index_second ... ok
test test_parse_array_literal ... ok
test test_parse_empty_array ... ok
test test_parse_array_with_variables ... ok
test test_ordinal_index_third ... ok
test test_pop_multiple ... ok
test test_pop_operation ... ok
test test_push_operation ... ok
test test_push_multiple ... ok
test test_string_contains ... ok
test test_string_join ... ok
test test_string_split ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 14 tests
test test_greek_numeral_word ... ok
test test_array_index_codegen ... ok
test test_hello_cosmos ... ok
test test_immutable_assignment_error ... ok
test test_assignment_missing_value_error ... ok
test test_assignment_codegen ... ok
test test_mutable_binding ... ok
test test_multiple_statements ... ok
test test_number_literal ... ok
test test_preserves_greek_in_strings ... ok
test test_mutable_binding_and_reassignment ... ok
test test_undefined_assignment_error ... ok
test test_variable_binding ... ok
test test_variable_binding_and_use ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 20 tests
test test_comparison_with_genitive ... ok
test test_continue ... ok
test test_break ... ok
test test_block_in_while_loop ... ok
test test_for_iteration ... ok
test test_for_range_exclusive ... ok
test test_ean_conditional ... ok
test test_lexicon_conditional_particles ... ok
test test_for_range_inclusive ... ok
test test_lexicon_else_particle ... ok
test test_lexicon_loop_control ... ok
test test_lexicon_loop_particles ... ok
test test_if_else ... ok
test test_lexicon_match_particle ... ok
test test_if_elif_else ... ok
test test_match_wildcard ... ok
test test_subjunctive_verb_form ... ok
test test_simple_if ... ok
test test_match_basic ... ok
test test_while_loop ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 1 test
test test_perfect_fold_coverage ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_perfect_passive_map_coverage ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 10 tests
test test_dangling_operator_ignored ... ok
test test_checked_arithmetic_codegen ... ok
test test_excess_operators_ignored ... ok
test test_dangling_propagation ... ok
test test_operator_only_ignored ... ok
test test_operator_without_subject_ignored ... ok
test test_standalone_subject_op_literal ... ok
test test_expression_propagation ... ok
test test_standalone_subject_op_nominative ... ok
test test_standalone_subject_op_object ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 10 tests
test test_codegen_function_keyword ... ok
test test_function_typed_params ... ok
test test_function_local_variables ... ok
test test_function_call ... ok
test test_parse_simple_function_no_params ... ok
test test_function_with_two_params ... ok
test test_nested_calls ... ok
test test_parameter_shadowing ... ok
test test_return_type_inference ... ok
test test_simple_return ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test test_huge_numeral_overflow_attempt ... ok
test fuzz_numerals ... ok
test fuzz_parser ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s


running 13 tests
test test_limit_arrays ... ok
test test_limit_blocks ... ok
test test_limit_genitives ... ok
test test_limit_index_accesses ... ok
test test_limit_literals ... ok
test test_limit_nested_phrases ... ok
test test_limit_participles ... ok
test test_limit_adjectives ... ok
test test_limit_nominatives ... ok
test test_limit_operators ... ok
test test_limit_unwraps ... ok
test test_limit_property_accesses ... ok
test test_limit_string_method_properties ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test test_limit_operators_or ... ok
test test_limit_operators_comparison ... ok
test test_limit_operators_arithmetic ... ok
test test_limit_ordinal_index ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 6 tests
test test_binop_recursion_coverage ... ok
test test_phrase_recursion_coverage ... ok
test test_binding_recursion_coverage ... ok
test test_call_recursion_coverage ... ok
test test_unaryop_recursion_coverage ... ok
test test_property_access_recursion_happy_path ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_disambiguate_nan_panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_dos_dev_zero ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_property_access_stack_overflow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_stack_overflow_mitigation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


running 1 test
test havoc_return_complex_expression - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 38 tests
test cycle12_aorist_participles::test_aorist_participle_detection ... ok
test cycle12_aorist_participles::test_aorist_participle_in_ast ... ok
test cycle12_aorist_participles::test_present_vs_aorist_capture_mode ... ok
test cycle11_variable_capture::test_no_capture_with_literal ... ok
test cycle11_variable_capture::test_capture_in_any ... ok
test cycle11_variable_capture::test_capture_in_filter ... ok
test cycle13_perfect_participles::test_memoization_codegen ... ok
test cycle13_perfect_participles::test_perfect_generates_memoized_closure ... ok
test cycle13_perfect_participles::test_perfect_participle_detection ... ok
test cycle13_perfect_participles::test_tense_capture_mode_mapping ... ok
test cycle1_participle_morphology::test_aorist_active_participle_feminine ... ok
test cycle1_participle_morphology::test_aorist_active_participle_masculine ... ok
test cycle1_participle_morphology::test_aorist_active_participle_neuter ... ok
test cycle1_participle_morphology::test_perfect_passive_participle ... ok
test cycle1_participle_morphology::test_present_active_participle ... ok
test cycle1_participle_morphology::test_present_middle_participle ... ok
test cycle1_participle_morphology::test_verb_lemma_extraction ... ok
test cycle4_map_operation::test_participle_word_detection ... ok
test cycle4_map_operation::test_map_with_participle_simple ... ok
test cycle5_filter_operation::test_comparative_adjective_detection ... ok
test cycle4_map_operation::test_map_with_participle ... ok
test cycle5_filter_operation::test_filter_less_than ... ok
test cycle6_find_operation::test_find_first_element ... ok
test cycle5_filter_operation::test_filter_with_comparative ... ok
test cycle6_find_operation::test_find_verb_detection ... ok
test cycle7_fold_operation::test_fold_verb_detection ... ok
test cycle6_find_operation::test_find_with_comparative ... ok
test cycle7_fold_operation::test_fold_with_product ... ok
test cycle8_any_all_operations::test_all_quantifier ... ok
test cycle7_fold_operation::test_fold_with_sum ... ok
test cycle8_any_all_operations::test_all_with_predicate ... ok
test cycle8_any_all_operations::test_any_less_than ... ok
test cycle9_combined_operations::test_filter_then_any ... ok
test cycle8_any_all_operations::test_any_quantifier_detection ... ok
test cycle8_any_all_operations::test_any_with_predicate ... ok
test cycle9_combined_operations::test_filter_then_map ... ok
test cycle9_combined_operations::test_map_then_fold ... ok
test cycle9_combined_operations::test_triple_chain ... ok

test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 17 tests
test test_bard_binding_mutable ... ok
test test_bard_assignment ... ok
test test_bard_break_continue ... ok
test test_bard_expression_stmt ... ok
test test_bard_function_def ... ok
test test_bard_exprs ... ok
test test_bard_print_multiple ... ok
test test_bard_for ... ok
test test_bard_query ... ok
test test_bard_match ... ok
test test_bard_return ... ok
test test_bard_test_decl ... ok
test test_bard_trait_def ... ok
test test_bard_trait_impl ... ok
test test_bard_type_def ... ok
test test_bard_types ... ok
test test_bard_while ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 9 tests
test test_adjective_normalization ... ok
test test_assertion_normalization ... ok
test test_assignment_normalization ... ok
test test_binding_normalization ... ok
test test_collection_ops_normalization ... ok
test test_equality_normalization ... ok
test test_print_normalization ... ok
test test_query_normalization ... ok
test test_participle_normalization_in_binding ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 6 tests
test test_arabic_fallback ... ok
test test_greek_numerals_array ... ok
test test_greek_numerals_index ... ok
test test_greek_numerals_assignment ... ok
test test_greek_numerals_mixed ... ok
test test_invalid_greek_logic ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 13 tests
test test_arithmetic_product ... ok
test test_arithmetic_difference ... ok
test test_arithmetic_quotient ... ok
test test_arithmetic_in_binding ... ok
test test_boolean_and ... ok
test test_boolean_not ... ok
test test_arithmetic_remainder ... ok
test test_arithmetic_sum ... ok
test test_boolean_or ... ok
test test_comparison_equal ... ok
test test_comparison_in_binding ... ok
test test_comparison_greater_than ... ok
test test_comparison_less_than ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 59 tests
test test_epitychia_is_ok ... ok
test test_err_codegen ... ok
test test_chained_propagation ... ok
test test_err_expression_exact_output ... ok
test test_err_expression_analyzed ... ok
test test_is_err_word_helper ... ok
test test_is_none_word_helper ... ok
test test_is_ok_word_helper ... ok
test test_is_some_word_helper ... ok
test test_err_with_different_types ... ok
test test_err_with_string_literal ... ok
test test_mixed_option_result ... ok
test test_nested_option ... ok
test test_none_codegen ... ok
test test_multiple_results_in_sequence ... ok
test test_none_expression_analyzed ... ok
test test_multiple_unwraps_sequence ... ok
test test_none_handling ... ok
test test_ok_expression_analyzed ... ok
test test_ok_codegen ... ok
test test_ok_expression_exact_output ... ok
test test_optative_aorist_passive ... ok
test test_optative_present_active ... ok
test test_option_to_rust ... ok
test test_option_type_compatibility ... ok
test test_option_type_exists ... ok
test test_option_none_generates_correctly ... ok
test test_option_in_print ... ok
test test_option_with_string ... ok
test test_ouden_is_none ... ok
test test_option_workflow ... ok
test test_propagation_early_return ... ok
test test_propagation_generates_question_mark ... ok
test test_propagation_operator_detection ... ok
test test_propagation_in_workflow ... ok
test test_propagation_vs_unwrap ... ok
test test_propagation_with_result ... ok
test test_realistic_error_handling ... ok
test test_result_ok_generates_correctly ... ok
test test_result_err_generates_correctly ... ok
test test_result_to_rust ... ok
test test_result_type_compatibility ... ok
test test_result_type_exists ... ok
test test_result_print_directly ... ok
test test_result_with_number_error ... ok
test test_result_unwrap ... ok
test test_result_parallels_option ... ok
test test_some_codegen ... ok
test test_result_with_number_value ... ok
test test_separate_option_and_variable ... ok
test test_some_expression_analyzed ... ok
test test_result_workflow ... ok
test test_sphalma_is_err ... ok
test test_some_with_different_types ... ok
test test_unwrap_in_expression ... ok
test test_unwrap_operator_codegen ... ok
test test_statement_end_with_semicolon ... ok
test test_ti_is_some ... ok
test test_unwrap_preserves_value ... ok

test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 1 test
test test_undefined_struct_type_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_file_size_limit_cli ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s


running 1 test
test test_security_memoize_repro - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test test_diacritic_only_variable_panic ... ok
test test_stack_overflow_nested_parens ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test test_length_property_not_ignored_without_subject ... ok
test test_ordinal_not_ignored_without_subject ... ok
test test_split_verb_consumes_literal_without_subject ... ok
test test_split_verb_not_ignored_without_delimiter ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test test_double_indirect_object_error ... ok
test test_neuter_plural_agreement ... ok
test test_verbless_statement ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test test_invalid_participles ... ok
test test_participle_analysis_coverage ... ok
test test_participle_lemma_generation ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_struct_instantiation_with_variable_args ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 6 tests
test test_empty_test_body ... ok
test test_assert_eq_compiles ... ok
test test_hashset_assertion ... ok
test test_assert_containment_compiles ... ok
test test_multiple_assertions_in_one_test ... ok
test test_test_function_name_sanitization ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test test_parse_test_declaration ... ok
test test_parse_test_with_body ... ok
test test_parse_multiple_tests ... ok
test test_parse_test_unaccented_keywords ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 32 tests
test test_analyze_trait_definition ... ok
test test_analyze_trait_impl ... ok
test test_call_default_method ... ok
test test_call_both_trait_methods_on_same_type ... ok
test test_call_trait_method ... ok
test test_call_trait_method_with_args ... ok
test test_codegen_full_example ... ok
test test_codegen_trait_definition ... ok
test test_default_method_body_analysis ... ok
test test_codegen_trait_with_default ... ok
test test_codegen_trait_impl ... ok
test test_codegen_trait_method_call_genitive ... ok
test test_duplicate_trait_error ... ok
test test_impl_for_undefined_trait_error ... ok
test test_impl_for_undefined_type_error ... ok
test test_missing_required_method_error ... ok
test test_impl_with_default_method_not_required ... ok
test test_parse_empty_trait ... ok
test test_parse_empty_trait_impl ... ok
test test_override_default_method ... ok
test test_multiple_types_implement_same_trait ... ok
test test_parse_trait_impl_with_method ... ok
test test_parse_trait_with_required_method ... ok
test test_parse_impl_multiple_methods ... ok
test test_parse_trait_with_default_method ... ok
test test_parse_trait_multiple_methods ... ok
test test_repro_trait_default_method_return_type ... ok
test test_trait_method_call_error_not_implemented ... ok
test test_repro_trait_impl_return_type ... ok
test test_trait_stored_in_scope ... ok
test test_standalone_method_call ... ok
test test_type_implements_multiple_traits ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 12 tests
test test_analyze_type_definition ... ok
test test_analyze_type_with_multiple_fields ... ok
test test_field_access ... ok
test test_field_access_multiple_fields ... ok
test test_instantiation ... ok
test test_instantiation_multiple_fields ... ok
test test_instantiation_with_explicit_numeric_word ... ok
test test_instantiation_with_literals ... ok
test test_instantiation_with_word_number ... ok
test test_instantiation_with_boolean ... ok
test test_parse_empty_type ... ok
test test_parse_type_with_field ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test test_coverage_find_patterns ... ok
test test_coverage_any_all_patterns ... ok
test test_coverage_filter_patterns ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_exploit_unicode_panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test test_cache_key_determinism ... ok
test test_cache_key_canonicalization ... ok
test test_cache_key_uniqueness ... ok
test test_cache_key_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test tests::test_trim_end_matches_bug_reproduction ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 9 tests
test test_article_disambiguation_context ... ok
test test_binding_word_order_independence ... ok
test test_multiple_statement_scope ... ok
test test_assembler_consistency ... ok
test test_print_word_order_independence ... ok
test test_query_produces_output ... ok
test test_number_binding_variations ... ok
test test_string_binding ... ok
test test_variable_binding_and_reference ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 37 tests
test src/codegen.rs - codegen (line 71) ... ignored
test src/codegen.rs - codegen::to_rust_type (line 219) ... ok
test src/ast.rs - ast::Word (line 366) ... ok
test src/errors/messages.rs - errors::messages::case_mismatch (line 98) ... ok
test src/codegen.rs - codegen::sanitize_name (line 134) ... ok
test src/errors/messages.rs - errors::messages::undefined_variable (line 23) ... ok
test src/errors/messages.rs - errors::messages::gender_mismatch (line 58) ... ok
test src/errors/messages.rs - errors::messages::immutable_assignment (line 39) ... ok
test src/errors/messages.rs - errors::messages::number_mismatch (line 78) ... ok
test src/grammar.rs - grammar (line 36) ... ok
test src/lib.rs - (line 50) ... ok
test src/morphology/mod.rs - morphology::analyses_compatible (line 218) ... ok
test src/morphology/conjugation.rs - morphology::conjugation::analyze_verb (line 313) ... ok
test src/morphology/participle.rs - morphology::participle::analyze_participle (line 494) ... ok
test src/morphology/participle.rs - morphology::participle (line 18) ... ok
test src/morphology/participle.rs - morphology::participle::ParticipleAnalysis::verb_lemma (line 54) ... ok
test src/morphology/mod.rs - morphology::analyze_all (line 113) ... ok
test src/parser.rs - parser::parse (line 45) ... ok
test src/parser/numerals.rs - parser::numerals::parse_greek_numeral (line 26) ... ok
test src/semantic/assembler.rs - semantic::assembler (line 74) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed (line 172) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed_array (line 302) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed_block (line 325) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed_index_access (line 371) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed_boolean (line 281) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed_nested_phrase (line 349) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed_participle (line 424) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed_number (line 260) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed_string (line 239) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::feed_unwrap (line 398) ... ok
test src/semantic/patterns.rs - semantic::patterns (line 20) ... ignored
test src/semantic/assembler.rs - semantic::assembler::Assembler::new (line 145) ... ok
test src/semantic/expressions.rs - semantic::expressions::analyze_argument_expr (line 28) ... ok
test src/semantic/types.rs - semantic::types::detect_collection_type (line 177) ... ok
test src/semantic/assembler.rs - semantic::assembler::Assembler::finalize (line 615) ... ok
test src/text.rs - text::normalize_greek (line 21) ... ok
test src/tools/highlight.rs - tools::highlight (line 21) ... ok

test result: ok. 35 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.03s

all doctests ran in 0.59s; merged doctests compilation took 0.56s compiles successfully.
**[Clippy Needless Borrow]**
**Learning:**  flags explicit references (e.g. ) when the value is already a reference (e.g.  is ) and the function takes that reference type. Rust auto-derefs  to , but Clippy prefers passing  directly.
**Action:** When a variable is already a reference (like ), pass it directly to functions expecting that reference type, instead of re-borrowing it.
**[Diff Coverage Traps]**
**Learning:** Codecov analyzes coverage *on the diff*. If you refactor a function (e.g., adding an argument), you touch lines that handle rare edge cases (like  or ). If existing tests don't hit those edge cases, the diff coverage drops, failing CI.
**Action:** When refactoring core logic (like a dispatcher or main match block), audit *all* branches to ensure at least one test case hits each branch, or add new tests to cover the gaps.
