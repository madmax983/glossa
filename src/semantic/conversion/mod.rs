//! Conversion from assembled statements to analyzed statements
//!
//! This module acts as the "interpreter" of the assembled semantic structure.
//! While the [`Assembler`](crate::semantic::Assembler) ensures grammatical correctness (Subject-Verb agreement),
//! this module assigns *meaning* to the grammatical structures.
//!
//! # The Interpreter Pattern
//!
//! The conversion process is essentially an interpretation step. It takes a
//! grammatically valid but semantically ambiguous "Assembled Statement" and
//! converts it into a typed, unambiguous "Analyzed Statement" (part of the HIR).
//!
//! This is where "word order independence" meets "semantic meaning".
//!
//! # Pattern Detection Strategy
//!
//! The [`classify_assembled_statement`] function uses a combination of strategies to
//! understand the statement's intent, checking patterns in a specific heuristic order:
//!
//! 1. **Pattern Delegation**: Complex patterns are delegated first.
//!    - **Iterator Chains**: `detect_iterator_pattern` (e.g., `list doubling print`).
//!    - **Property Access**: `classify_property_access_print` (e.g., `user.name print`).
//!    - **Struct Instantiation**: `try_parse_struct_instantiation` (e.g., `x new User ... let`).
//!    - **Function Calls**: `classify_function_call` (e.g., `my_func arg1 arg2 call`).
//!
//! 2. **Verb-Based Classification**: If no complex pattern matches, the main verb drives the logic.
//!    - **Binding** (`ἔστω`): `let x = value`.
//!    - **Assignment** (`γίγνεται`): `x = value`.
//!    - **Collection Ops** (`ὠθεῖ`, `ἕλκεται`, `τίθησι`): `push`, `pop`, `insert`.
//!    - **Print** (`λέγε`, `γράφε`): `println!`.
//!    - **Query** (`?`): Expressions ending in `?`.
//!
//! 3. **Expression Fallback**: If no verb implies a statement, it's treated as a pure expression.
//!    - **Operations**: `1 + 2`.
//!    - **Try/Propagate**: `expr;` (becomes `expr?`).

pub(crate) mod classification;
pub(crate) mod extraction;
pub use classification::*;
pub use extraction::*;

use super::expressions::{
    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    literal_to_analyzed_expr,
};
use super::patterns::detect_iterator_pattern;
use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::morphology::{self};
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::model::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};
use crate::semantic::resolver::Scope;

use crate::semantic::types::GlossaType;
use crate::semantic::{Constituent, Literal};

/// Convert an AssembledStatement to an AnalyzedStatement
///
/// This is the main entry point for lowering the "Assembled" semantic model (slot-based)
/// to the "Analyzed" model (HIR/AST-like).
///
/// Evaluates and translates a grammatically sound statement into the semantically typed AST.
///
/// This serves as the top-level interpreter connecting the raw output of the
/// [`crate::semantic::Assembler`] (`AssembledStatement`) with the High-Level Intermediate
/// Representation (`AnalyzedStatement`). It assigns concrete meaning to grammatical roles
/// (e.g., assigning a Subject the role of "Variable Name").
///
/// # Arguments
///
/// * `asm_stmt` - The assembled statement from the `Assembler`.
/// * `scope` - The current semantic scope (for variable lookup and definition).
///
/// # Examples
///
/// ```rust,ignore
/// // Example cannot be run as a doctest because this module is pub(crate)
/// use glossa::semantic::assembly::AssembledStatement;
/// use glossa::semantic::conversion::convert_assembled_to_analyzed;
/// use glossa::semantic::resolver::Scope;
/// use glossa::ast::{Expr, Word};
/// use glossa::morphology::lexicon::{LexiconEntry, VerbType};
///
/// let mut scope = Scope::new();
/// let mut asm = AssembledStatement::new();
///
/// // Simulate: "«χαῖρε» λέγε."
/// asm.verb = Some(LexiconEntry {
///     lemma: "λεγω".into(),
///     english_equivalent: "say".into(),
///     part_of_speech: glossa::morphology::lexicon::PartOfSpeech::Verb(VerbType::Transitive),
/// });
/// asm.strings.push("χαῖρε".into());
///
/// let result = convert_assembled_to_analyzed(&asm, &mut scope);
/// assert!(result.is_ok());
/// ```
pub fn convert_assembled_to_analyzed(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    classify_assembled_statement(asm_stmt, scope)
}


// -------------------------------------------------------------------------------------------------
// Helper functions for classify_assembled_statement
// -------------------------------------------------------------------------------------------------






/// Helper: Resolve the target variable name and the effective assembled statement for binding
///
/// ⚡ Bolt Optimization: Returns a `std::borrow::Cow<'_, AssembledStatement>` instead of
/// `AssembledStatement` to avoid cloning a large struct on a hot path during semantic analysis.
/// We only need to clone and mutate the assembled statement if we're swapping subject/object
/// or fixing false participles. Otherwise, we just return a borrowed reference to the original statement.
fn resolve_binding_target<'a>(
    asm_stmt: &'a AssembledStatement,
    scope: &Scope,
) -> Result<(String, std::borrow::Cow<'a, AssembledStatement>), GlossaError> {
    // Check for "false participles" (nouns misclassified as participles)
    let has_false_participle = !asm_stmt.participles.is_empty()
        && morphology::lexicon::lookup(&asm_stmt.participles[0].verb_lemma).is_none();

    if has_false_participle {
        let first_participle = &asm_stmt.participles[0];
        let mut fixed_asm = asm_stmt.clone();
        fixed_asm.participles = asm_stmt.participles[1..].to_vec();
        return Ok((
            first_participle.normalized.to_string(),
            std::borrow::Cow::Owned(fixed_asm),
        ));
    }

    // Check for Subject/Object swap (if Subject is defined and Object is not, bind to Object)
    if let (Some(subject), Some(object)) = (&asm_stmt.subject, &asm_stmt.object) {
        let subject_name = &subject.normalized;
        let object_name = &object.normalized;

        if scope.is_defined(&subject.lemma) && !scope.is_defined(&object.lemma) {
            let mut swapped = asm_stmt.clone();
            swapped.subject = Some(object.clone());
            swapped.object = Some(subject.clone());
            return Ok((object_name.to_string(), std::borrow::Cow::Owned(swapped)));
        } else {
            return Ok((
                subject_name.to_string(),
                std::borrow::Cow::Borrowed(asm_stmt),
            ));
        }
    }

    // Default case: Bind to Subject
    if let Some(subject) = &asm_stmt.subject {
        return Ok((
            subject.normalized.to_string(),
            std::borrow::Cow::Borrowed(asm_stmt),
        ));
    }

    // Fallback: Bind to first participle (if any remain)
    if !asm_stmt.participles.is_empty() {
        let first_participle = &asm_stmt.participles[0];
        let mut fixed_asm = asm_stmt.clone();
        fixed_asm.participles = asm_stmt.participles[1..].to_vec();
        return Ok((
            first_participle.normalized.to_string(),
            std::borrow::Cow::Owned(fixed_asm),
        ));
    }

    Err(GlossaError::semantic("Binding without subject"))
}





















// -------------------------------------------------------------------------------------------------
// Helper functions for extract_value
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_pop_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(), // not a pop verb
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_pop("λέγει", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_pop_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ὠθεῖ".into(), // actually a push verb, let's use ἕλκεται for pop, but any string works for the missing subject test since it checks lemma first
                normalized: "ἕλκεται".into(),
                original: "ἕλκεται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        // The check inside classify_pop explicitly looks at the passed verb_lemma ("ἕλκεται" is pop)
        let result = classify_pop("ἕλκεται", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_push_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_push("λέγει", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_push_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ὠθεῖ".into(),
                normalized: "ὠθεῖ".into(),
                original: "ὠθεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_push("ὠθεῖ", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_insert_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("λέγει", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_insert_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "τίθησι".into(),
                normalized: "τίθησι".into(),
                original: "τίθησι".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("τίθησι", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_no_containment() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "δεῖ".into(),
                normalized: "δεῖ".into(),
                original: "δεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            has_containment_preposition: false,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "δεῖ".into(),
                normalized: "δεῖ".into(),
                original: "δεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            has_containment_preposition: true,
            subject: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_missing_left_expr() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἰσοῦται".into(),
                normalized: "ἰσοῦται".into(),
                original: "ἰσοῦται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None, // Missing subject means left_expr will be None
            literals: vec![Literal::Number(5)],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_undefined_left_expr() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἰσοῦται".into(),
                normalized: "ἰσοῦται".into(),
                original: "ἰσοῦται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "y".into(), // y is not defined in scope
                normalized: "y".into(),
                original: "y".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![Literal::Number(5)],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_missing_right_expr() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἰσοῦται".into(),
                normalized: "ἰσοῦται".into(),
                original: "ἰσοῦται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![], // Empty literals means right_expr will be None
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::Number);
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_collection_mutation_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_collection_mutation(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_print_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_print_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(), // not a print verb (λέγε is, but here testing the literal check)
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_print_binary_op_empty() {
        let asm_stmt = AssembledStatement {
            operators: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_binary_op(&asm_stmt, &mut scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_print_property_access_empty() {
        let asm_stmt = AssembledStatement {
            property_accesses: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_property_access(&asm_stmt, &mut scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_print_index_access_empty() {
        let asm_stmt = AssembledStatement {
            index_accesses: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_index_access(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_print_unwrap_empty() {
        let asm_stmt = AssembledStatement {
            unwraps: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_unwrap(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_no_subject() {
        let asm_stmt = AssembledStatement {
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_no_genitives() {
        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "len".into(),
                normalized: "len".into(),
                original: "len".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            genitives: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_owner_not_found() {
        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "len".into(),
                normalized: "len".into(),
                original: "len".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            genitives: vec![Constituent {
                lemma: "x".into(), // Not in scope
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_method_already_defined() {
        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "len".into(),
                normalized: "len".into(),
                original: "len".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            genitives: vec![Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::String);
        scope.define("len", GlossaType::Number); // Method name is already a defined variable in scope
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_genitive_method_call_empty() {
        let asm_stmt = AssembledStatement {
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_genitive_method_call(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_unwrap_empty() {
        let asm_stmt = AssembledStatement {
            unwraps: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_unwrap(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_enum_from_subject_empty() {
        let asm_stmt = AssembledStatement {
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_enum_from_subject(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_genitive_method_empty() {
        let asm_stmt = AssembledStatement {
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_genitive_method(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_enum_from_nominatives_empty() {
        let asm_stmt = AssembledStatement {
            nominatives: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_enum_from_nominatives(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_property_access_empty() {
        let asm_stmt = AssembledStatement {
            property_accesses: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_property_access(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_index_access_empty() {
        let asm_stmt = AssembledStatement {
            index_accesses: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_index_access(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_array_empty() {
        let asm_stmt = AssembledStatement {
            arrays: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_array(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_binary_op_empty() {
        let asm_stmt = AssembledStatement {
            operators: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_binary_op(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_enum_from_object_empty() {
        let asm_stmt = AssembledStatement {
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_enum_from_object(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_literal_empty() {
        let asm_stmt = AssembledStatement {
            literals: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_literal(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_object_fallback_empty() {
        let asm_stmt = AssembledStatement {
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_object_fallback(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_property_access_print_owner_not_in_scope() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγε".into(),
                normalized: "λέγε".into(),
                original: "λέγε".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            genitives: vec![Constituent {
                lemma: "owner".into(),
                normalized: "owner".into(),
                original: "owner".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            subject: Some(Constituent {
                lemma: "prop".into(),
                normalized: "prop".into(),
                original: "prop".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_property_access_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_property_access_print_owner_not_struct() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγε".into(),
                normalized: "λέγε".into(),
                original: "λέγε".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            genitives: vec![Constituent {
                lemma: "owner".into(),
                normalized: "owner".into(),
                original: "owner".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            subject: Some(Constituent {
                lemma: "prop".into(),
                normalized: "prop".into(),
                original: "prop".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("owner", GlossaType::Number); // Not a struct
        let result = classify_property_access_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_function_call_no_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἔστω".into(), // Binding verb
                normalized: "ἔστω".into(),
                original: "ἔστω".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            object: Some(Constituent {
                lemma: "myfunc".into(),
                normalized: "myfunc".into(),
                original: "myfunc".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: None,
                person: None,
            }),
            subject: None, // No subject
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define_function("myfunc", vec![], Some(GlossaType::Number));
        let result = classify_function_call(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_subjunctive_comparison_no_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἔστω".into(), // Binding verb
                normalized: "ἔστω".into(),
                original: "ἔστω".into(),
                person: None,
                number: None,
                tense: None,
                mood: Some(crate::morphology::Mood::Subjunctive),
                voice: None,
            }),
            operators: vec![crate::morphology::lexicon::BinaryOp::Eq],
            literals: vec![Literal::Number(5)],
            subject: None, // No subject
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_subjunctive_comparison(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_resolve_binding_target_subject_object_swap() {
        // Create a scope where 'subject_var' IS defined but 'object_var' is NOT defined.
        // This should trigger the Subject/Object swap logic.
        let mut scope = Scope::new();
        scope.define("subject_var", GlossaType::Number);

        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "subject_var".into(),
                normalized: "subject_var".into(),
                original: "subject_var".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            object: Some(Constituent {
                lemma: "object_var".into(),
                normalized: "object_var".into(),
                original: "object_var".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            ..Default::default()
        };

        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        // Since 'subject_var' was defined and 'object_var' was not, it should bind to 'object_var'
        assert_eq!(name, "object_var");
        assert!(matches!(fixed_asm, std::borrow::Cow::Owned(_)));

        // Ensure they were actually swapped
        assert_eq!(fixed_asm.subject.as_ref().unwrap().lemma, "object_var");
        assert_eq!(fixed_asm.object.as_ref().unwrap().lemma, "subject_var");
    }

    #[test]
    fn test_resolve_binding_target_subject_object_no_swap() {
        // Create a scope where NEITHER is defined.
        // This should skip the swap logic and fall into the 'else' branch,
        // binding to the subject and returning Cow::Borrowed.
        let scope = Scope::new();

        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "subject_var".into(),
                normalized: "subject_var".into(),
                original: "subject_var".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            object: Some(Constituent {
                lemma: "object_var".into(),
                normalized: "object_var".into(),
                original: "object_var".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            ..Default::default()
        };

        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        assert_eq!(name, "subject_var");
        assert!(matches!(fixed_asm, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn test_resolve_binding_target_no_subject_has_participle() {
        // This tests the "Fallback: Bind to first participle (if any remain)" case
        // We use a verb_lemma that exists in the lexicon so it's NOT treated as a "false participle"
        // Let's use "λεγω" which is definitely a verb
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![crate::semantic::assembly::ParticipleConstituent {
                verb_lemma: "λεγω".into(),
                normalized: "λεγων".into(), // Actual participle
                original: "λέγων".into(),
                gender: crate::morphology::Gender::Masculine,
                case: crate::morphology::Case::Nominative,
                number: crate::morphology::Number::Singular,
                voice: crate::morphology::Voice::Active,
                tense: crate::morphology::Tense::Present,
            }],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        assert_eq!(name, "λεγων");
        assert!(matches!(fixed_asm, std::borrow::Cow::Owned(_)));
        assert!(fixed_asm.participles.is_empty()); // Should have been consumed
    }

    #[test]
    fn test_resolve_binding_target_false_participle() {
        // This tests the "false participle" check at the very beginning of the function
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![crate::semantic::assembly::ParticipleConstituent {
                verb_lemma: "not_a_real_verb_lemma".into(), // Will fail lexicon lookup
                normalized: "false_participle".into(),
                original: "false_participle".into(),
                gender: crate::morphology::Gender::Masculine,
                case: crate::morphology::Case::Nominative,
                number: crate::morphology::Number::Singular,
                voice: crate::morphology::Voice::Active,
                tense: crate::morphology::Tense::Present,
            }],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        assert_eq!(name, "false_participle");
        assert!(matches!(fixed_asm, std::borrow::Cow::Owned(_)));
    }

    #[test]
    fn test_resolve_binding_target_no_subject_no_participle() {
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Binding without subject")
        );
    }

    #[test]
    fn test_classify_query_containment_no_literal() {
        let asm_stmt = AssembledStatement {
            is_query: true,
            has_containment_preposition: true,
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![], // No literal element
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::List(Box::new(GlossaType::Number)));
        let result = classify_query(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        let stmt = result.unwrap();
        assert!(stmt.is_some());

        // Ensure the fallback literal generation (0) happened
        if let AnalyzedStatement::Query(exprs) = stmt.unwrap() {
            assert_eq!(exprs.len(), 1);
            if let AnalyzedExprKind::MethodCall { args, .. } = &exprs[0].expr {
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::UnaryOp { op, operand } = &args[0].expr {
                    assert_eq!(*op, crate::morphology::lexicon::UnaryOp::Ref);
                    assert!(matches!(operand.expr, AnalyzedExprKind::NumberLiteral(0)));
                } else {
                    panic!("Expected UnaryOp Ref");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Query");
        }
    }

    #[test]
    fn test_classify_insert_no_args() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "τιθημι".into(), // insert verb
                normalized: "τιθημι".into(),
                original: "τίθησι".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![],
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("τιθημι", &asm_stmt, &scope);
        assert!(result.is_ok());
        let opt_stmt = result.unwrap();
        assert!(opt_stmt.is_some());
        if let AnalyzedStatement::Expression(exprs) = opt_stmt.unwrap() {
            if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
                assert_eq!(method, "insert");
                assert!(args.is_empty());
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_classify_insert_object() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "τιθημι".into(), // insert verb
                normalized: "τιθημι".into(),
                original: "τίθησι".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            object: Some(Constituent {
                lemma: "y".into(),
                normalized: "y".into(),
                original: "y".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: None,
                person: None,
            }),
            literals: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("τιθημι", &asm_stmt, &scope);
        assert!(result.is_ok());
        let opt_stmt = result.unwrap();
        assert!(opt_stmt.is_some());
        if let AnalyzedStatement::Expression(exprs) = opt_stmt.unwrap() {
            if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
                assert_eq!(method, "insert");
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::Variable(var_name) = &args[0].expr {
                    assert_eq!(var_name, "y");
                } else {
                    panic!("Expected Variable argument");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_classify_push_no_args() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ωθω".into(), // push verb
                normalized: "ωθω".into(),
                original: "ὠθεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![],
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_push("ωθω", &asm_stmt, &scope);
        assert!(result.is_ok());
        let opt_stmt = result.unwrap();
        assert!(opt_stmt.is_some());
        if let AnalyzedStatement::Expression(exprs) = opt_stmt.unwrap() {
            if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
                assert_eq!(method, "push");
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::NumberLiteral(val) = args[0].expr {
                    assert_eq!(val, 0); // fallback is 0
                } else {
                    panic!("Expected NumberLiteral fallback argument");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_classify_expression_empty_exprs_propagate() {
        let scope = Scope::new();
        // Create an AssembledStatement that will produce an empty `exprs` array
        // but has `is_propagate` set to true.
        let asm_stmt = AssembledStatement {
            is_propagate: true,
            ..Default::default()
        };
        // No literals, operators, subject, object, or nested phrases -> exprs will be empty.

        let result = classify_expression(&asm_stmt, &scope);
        assert!(result.is_ok());

        if let AnalyzedStatement::Expression(exprs) = result.unwrap() {
            assert!(exprs.is_empty(), "Expected empty expressions array");
        } else {
            panic!("Expected AnalyzedStatement::Expression");
        }
    }
}
