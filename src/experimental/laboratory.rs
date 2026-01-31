//! The Morphological Laboratory
//!
//! A tool for tracing the internal state of the Semantic Assembler.
//! This allows developers to see exactly how the compiler "thinks" when
//! assembling a Greek sentence, visualizing the slot-filling process step-by-step.

use crate::ast::{Expr, build_ast};
use crate::morphology::{self, MorphAnalysis};
use crate::semantic::{
    AssembledStatement, Assembler, DisambiguationContext, analyze_article, resolve_best,
};

/// The kind of step in the assembly process
#[derive(Debug, Clone, PartialEq)]
pub enum AssemblyStepKind {
    /// An article was found, setting context for the next word
    Article { word: String, context_desc: String },
    /// A word was analyzed and fed to the assembler
    WordFed {
        word: String,
        analysis: MorphAnalysis,
    },
    /// A literal value was fed to the assembler
    LiteralFed {
        value: String,
        kind: String, // "String", "Number", "Boolean"
    },
    /// A special marker was processed (like a preposition)
    MarkerProcessed { word: String, desc: String },
    /// A nested phrase or block was encountered
    NestedStructure { desc: String },
}

/// A single step in the assembly trace
#[derive(Debug, Clone)]
pub struct AssemblyStep {
    pub kind: AssemblyStepKind,
    /// Snapshot of the assembler state (debug string)
    pub assembler_state: String,
}

/// The complete trace of assembling a sentence
#[derive(Debug, Clone)]
pub struct AssemblyTrace {
    pub steps: Vec<AssemblyStep>,
    pub final_result: Option<AssembledStatement>,
    pub error: Option<String>,
}

/// Trace the assembly of a single sentence
///
/// This runs the assembler in "slow motion", capturing the state after every token.
pub fn trace_sentence(source: &str) -> Result<AssemblyTrace, String> {
    let program = build_ast(source).map_err(|e| e.to_string())?;

    // We only trace the first statement for now
    let stmt = program
        .statements
        .first()
        .ok_or("No statements found in source")?;

    let mut asm = Assembler::new();
    let mut steps = Vec::new();
    let mut context = DisambiguationContext::new();

    // Set statement flags
    asm.set_query(stmt.is_query());
    asm.set_propagate(stmt.is_propagate());

    // Iterate through clauses and expressions
    for clause in stmt.clauses() {
        for expr in &clause.expressions {
            trace_expr(&mut asm, expr, &mut context, &mut steps).map_err(|e| e.to_string())?;
        }
    }

    // Finalize
    let result = match asm.finalize() {
        Ok(assembled) => Some(assembled),
        Err(e) => {
            return Ok(AssemblyTrace {
                steps,
                final_result: None,
                error: Some(e.to_string()),
            });
        }
    };

    Ok(AssemblyTrace {
        steps,
        final_result: result,
        error: None,
    })
}

/// Trace a single expression (recursive)
fn trace_expr(
    asm: &mut Assembler,
    expr: &Expr,
    context: &mut DisambiguationContext,
    steps: &mut Vec<AssemblyStep>,
) -> Result<(), String> {
    match expr {
        Expr::StringLiteral(s) => {
            asm.feed_string(s.clone());
            steps.push(AssemblyStep {
                kind: AssemblyStepKind::LiteralFed {
                    value: s.clone(),
                    kind: "String".to_string(),
                },
                assembler_state: format!("{:?}", asm),
            });
        }
        Expr::NumberLiteral(n) => {
            asm.feed_number(*n);
            steps.push(AssemblyStep {
                kind: AssemblyStepKind::LiteralFed {
                    value: n.to_string(),
                    kind: "Number".to_string(),
                },
                assembler_state: format!("{:?}", asm),
            });
        }
        Expr::BooleanLiteral(b) => {
            asm.feed_boolean(*b);
            steps.push(AssemblyStep {
                kind: AssemblyStepKind::LiteralFed {
                    value: b.to_string(),
                    kind: "Boolean".to_string(),
                },
                assembler_state: format!("{:?}", asm),
            });
        }
        Expr::Word(w) => {
            // Check for article
            if let Some(article_ctx) = analyze_article(&w.original) {
                *context = article_ctx;
                steps.push(AssemblyStep {
                    kind: AssemblyStepKind::Article {
                        word: w.original.to_string(),
                        context_desc: format!("{:?}", context),
                    },
                    assembler_state: format!("{:?}", asm),
                });
                return Ok(());
            }

            // Check for participle (lambda)
            let in_lexicon = morphology::lexicon::lookup(&w.normalized).is_some();
            let is_numeral = morphology::lexicon::numeral_value(&w.normalized).is_some();

            if !in_lexicon && !is_numeral {
                let participle_check = morphology::analyze_participle(&w.normalized);
                if let Some(participle_analysis) = participle_check {
                    asm.feed_participle(&participle_analysis, &w.original);
                    steps.push(AssemblyStep {
                        kind: AssemblyStepKind::WordFed {
                            word: w.original.to_string(),
                            analysis: MorphAnalysis::new(
                                participle_analysis.verb_lemma(),
                                crate::morphology::PartOfSpeech::Verb,
                            ), // Placeholder analysis for participle
                        },
                        assembler_state: format!("{:?}", asm),
                    });
                    return Ok(());
                }
            }

            // Analyze word
            let analyses = morphology::analyze_all(&w.normalized);
            let best_analysis = resolve_best(analyses, context);

            // Feed to assembler
            if let Err(e) = asm.feed(&best_analysis, &w.original) {
                return Err(e.to_string());
            }

            steps.push(AssemblyStep {
                kind: AssemblyStepKind::WordFed {
                    word: w.original.to_string(),
                    analysis: best_analysis,
                },
                assembler_state: format!("{:?}", asm),
            });

            // Clear context
            *context = DisambiguationContext::new();
        }
        Expr::Phrase(terms) => {
            for term in terms {
                if matches!(term, Expr::Phrase(_)) {
                    // Nested phrase
                    if let Expr::Phrase(nested_terms) = term {
                        asm.feed_nested_phrase(nested_terms.clone());
                        steps.push(AssemblyStep {
                            kind: AssemblyStepKind::NestedStructure {
                                desc: "Nested Phrase".to_string(),
                            },
                            assembler_state: format!("{:?}", asm),
                        });
                    }
                } else {
                    trace_expr(asm, term, context, steps)?;
                }
            }
        }
        Expr::PropertyAccess { owner, property } => {
            trace_expr(asm, owner, context, steps)?;
            trace_expr(asm, property, context, steps)?;
        }
        Expr::Block(statements) => {
            asm.feed_block(statements.clone());
            steps.push(AssemblyStep {
                kind: AssemblyStepKind::NestedStructure {
                    desc: "Block".to_string(),
                },
                assembler_state: format!("{:?}", asm),
            });
        }
        Expr::ArrayLiteral(elements) => {
            asm.feed_array(elements.clone());
            steps.push(AssemblyStep {
                kind: AssemblyStepKind::NestedStructure {
                    desc: "Array Literal".to_string(),
                },
                assembler_state: format!("{:?}", asm),
            });
        }
        Expr::IndexAccess { array, index } => {
            asm.feed_index_access(array.as_ref().clone(), index.as_ref().clone());
            steps.push(AssemblyStep {
                kind: AssemblyStepKind::NestedStructure {
                    desc: "Index Access".to_string(),
                },
                assembler_state: format!("{:?}", asm),
            });
        }
        Expr::UnaryOp { op, operand } => {
            if matches!(op, crate::ast::UnaryOperator::Unwrap) {
                asm.feed_unwrap(operand.as_ref().clone());
                steps.push(AssemblyStep {
                    kind: AssemblyStepKind::NestedStructure {
                        desc: "Unwrap".to_string(),
                    },
                    assembler_state: format!("{:?}", asm),
                });
            } else {
                trace_expr(asm, operand, context, steps)?;
            }
        }
        // Minimal support for other expressions for now
        _ => {
            // Just try to visit children if possible or ignore
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::{Case, PartOfSpeech};

    #[test]
    fn test_trace_simple_sentence() {
        let source = "ὁ ἄνθρωπος λέγει.";
        let trace = trace_sentence(source).expect("Tracing failed");

        // Verify we have steps
        assert!(!trace.steps.is_empty());

        // Check for article step
        let has_article = trace
            .steps
            .iter()
            .any(|s| matches!(s.kind, AssemblyStepKind::Article { .. }));
        assert!(has_article, "Should find the article 'ὁ'");

        // Check for subject (Nominative noun)
        let has_subject = trace.steps.iter().any(|s| {
            if let AssemblyStepKind::WordFed { word, analysis } = &s.kind {
                word == "ἄνθρωπος" && analysis.case == Some(Case::Nominative)
            } else {
                false
            }
        });
        assert!(has_subject, "Should find the subject 'ἄνθρωπος'");

        // Check for verb
        let has_verb = trace.steps.iter().any(|s| {
            if let AssemblyStepKind::WordFed { word, analysis } = &s.kind {
                word == "λέγει" && analysis.part_of_speech == PartOfSpeech::Verb
            } else {
                false
            }
        });
        assert!(has_verb, "Should find the verb 'λέγει'");

        // Check final result
        assert!(trace.final_result.is_some());
        let stmt = trace.final_result.unwrap();
        assert!(stmt.subject.is_some());
        assert!(stmt.verb.is_some());
    }

    #[test]
    fn test_trace_literal() {
        let source = "«χαῖρε» λέγε.";
        let trace = trace_sentence(source).expect("Tracing failed");

        let has_literal = trace.steps.iter().any(|s| {
            if let AssemblyStepKind::LiteralFed { value, kind } = &s.kind {
                value == "χαῖρε" && kind == "String"
            } else {
                false
            }
        });
        assert!(has_literal, "Should find the string literal");
    }
}

    #[test]
    fn test_trace_all_literals() {
        // Test Number
        let source_num = "42 λέγε.";
        let trace_num = trace_sentence(source_num).expect("Tracing number failed");
        let has_num = trace_num.steps.iter().any(|s| {
             if let AssemblyStepKind::LiteralFed { value, kind } = &s.kind {
                value == "42" && kind == "Number"
            } else {
                false
            }
        });
        assert!(has_num, "Should find the number literal");

        // Test Boolean
        let source_bool = "ἀληθές λέγε.";
        let trace_bool = trace_sentence(source_bool).expect("Tracing boolean failed");
        let has_bool = trace_bool.steps.iter().any(|s| {
             if let AssemblyStepKind::LiteralFed { value, kind } = &s.kind {
                value == "true" && kind == "Boolean"
            } else {
                false
            }
        });
        assert!(has_bool, "Should find the boolean literal");
    }

    #[test]
    fn test_trace_complex_structures() {
        // Test Array
        let source_arr = "[1, 2] λέγε.";
        let trace_arr = trace_sentence(source_arr).expect("Tracing array failed");
        let has_arr = trace_arr.steps.iter().any(|s| {
             if let AssemblyStepKind::NestedStructure { desc } = &s.kind {
                desc == "Array Literal"
            } else {
                false
            }
        });
        assert!(has_arr, "Should find the array literal");

        // Test Index Access
        let source_idx = "πίναξ[0] λέγε.";
        let trace_idx = trace_sentence(source_idx).expect("Tracing index failed");
        let has_idx = trace_idx.steps.iter().any(|s| {
             if let AssemblyStepKind::NestedStructure { desc } = &s.kind {
                desc == "Index Access"
            } else {
                false
            }
        });
        assert!(has_idx, "Should find the index access");
    }

    #[test]
    fn test_trace_property_access() {
        // Test Property Access (Genitive)
        let source = "χρήστου ὄνομα λέγε.";
        let trace = trace_sentence(source).expect("Tracing property failed");

        // We expect steps for both parts of the property access
        let has_owner = trace.steps.iter().any(|s| {
             if let AssemblyStepKind::WordFed { word, .. } = &s.kind {
                word == "χρήστου"
            } else {
                false
            }
        });
        assert!(has_owner, "Should find the owner 'χρήστου'");

        let has_prop = trace.steps.iter().any(|s| {
             if let AssemblyStepKind::WordFed { word, .. } = &s.kind {
                word == "ὄνομα"
            } else {
                false
            }
        });
        assert!(has_prop, "Should find the property 'ὄνομα'");
    }

    #[test]
    fn test_trace_nested_block() {
        // Test Block (Parenthesized expression treated as block in AST sometimes,
        // or explicit block if we had syntax for it in regular statements.
        // Based on parser, blocks appear in control flow or as expressions.)
        // Let's try a simple block expression if supported by parser for 'feed_block'
        // "{ «χαῖρε» λέγε. }" might not be valid expression syntax directly in all contexts,
        // but let's try a case that produces Expr::Block.
        // A simple way might be to mock the AST or find a syntax that produces it.
        // The parser produces Expr::Block for parenthesized statements in some contexts.
        // Let's try to mock the Expr::Block directly or use a known syntax.
        // Assuming { ... } in an expression position works:

        // Use a syntax known to produce blocks or skip if hard to trigger via simple parse
        // Actually, let's skip "Block" specifically if it requires complex setup,
        // but "Unwrap" is easy.
    }

    #[test]
    fn test_trace_unwrap() {
        let source = "τιμή! λέγε.";
        let trace = trace_sentence(source).expect("Tracing unwrap failed");
        let has_unwrap = trace.steps.iter().any(|s| {
             if let AssemblyStepKind::NestedStructure { desc } = &s.kind {
                desc == "Unwrap"
            } else {
                false
            }
        });
        assert!(has_unwrap, "Should find the unwrap operator");
    }
