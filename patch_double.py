import re

with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

# Let's see what I printed earlier. I printed the `dummy6.rs` output!
# AssembledStatement {
#     subject: Some(Constituent { lemma: "ξ", ... }),
#     nominatives: [],
#     verb: Some(VerbConstituent { lemma: "λεγω", ... }),
#     ...
#     genitives: [Constituent { lemma: "θος", original: "θου", ... }],
#     adjectives: [Constituent { lemma: "μεγας", original: "μείζονα", ... }]
# }
# Wait! In `dummy6.rs` output, `nominatives` WAS EMPTY!
# IF `nominatives` WAS EMPTY, WHY DID IT PANIC WITH DoubleSubject??
# "thread 'test_parse_filter' (95661) panicked at tests/dummy6.rs:7:5:"
# Wait, `dummy6.rs` panicked with `Print output` because I explicitly put `panic!("Print output")` there to print the AST!
# Oh!! `dummy6` DID NOT panic with DoubleSubject! `dummy6` succeeded parsing!
# So `assemble_statement` DID NOT throw DoubleSubject for `ξ θου μείζονα λέγε.`!!
# It returned the `AssembledStatement` successfully!
# THEN `test_coverage_filter_patterns` failed with `AssemblyError(DoubleSubject)`!
# WHY?!
# Let me look at `test_coverage_filter_patterns` again.
