//! The Rhetorician: A stylistic analyzer for ΓΛΩΣΣΑ
//!
//! "Words have weight. Code has rhythm."
//!
//! This module analyzes the *rhetorical style* of a GLOSSA program.
//! It looks at verb tenses, moods, and voices to determine the "tone"
//! of the code (e.g., Imperative/Commanding vs Indicative/Descriptive).

use crate::ast::{Expr, Program, Statement};
use crate::morphology::{Mood, PartOfSpeech, Tense, Voice, analyze};
use std::collections::HashMap;

/// Rhetorical statistics for a program
#[derive(Debug, Default, Clone, PartialEq)]
pub struct RhetoricalStats {
    /// Total number of words analyzed
    pub total_words: usize,
    /// Total number of verbs found
    pub total_verbs: usize,

    /// Count of tenses (Present = Continuous, Aorist = Immediate, etc.)
    pub tense_counts: HashMap<Tense, usize>,
    /// Count of moods (Indicative, Imperative, etc.)
    pub mood_counts: HashMap<Mood, usize>,
    /// Count of voices (Active, Passive, Middle)
    pub voice_counts: HashMap<Voice, usize>,

    /// "Eloquence Score" (0.0 to 100.0)
    /// A metric of vocabulary richness and syntactic variety.
    pub eloquence_score: f32,

    /// The dominant "tone" of the code
    pub tone: String,
}

/// Analyze the rhetorical style of a program
pub fn analyze_rhetoric(program: &Program) -> RhetoricalStats {
    let mut stats = RhetoricalStats::default();
    let mut unique_lemmas = std::collections::HashSet::new();

    // Traverse the AST
    for stmt in &program.statements {
        analyze_statement(stmt, &mut stats, &mut unique_lemmas);
    }

    // Calculate scores
    calculate_derived_stats(&mut stats, unique_lemmas.len());

    stats
}

fn analyze_statement(
    stmt: &Statement,
    stats: &mut RhetoricalStats,
    unique_lemmas: &mut std::collections::HashSet<String>,
) {
    // Flatten all expressions in the statement
    for expr in stmt.expressions() {
        analyze_expression(expr, stats, unique_lemmas);
    }
}

fn analyze_expression(
    expr: &Expr,
    stats: &mut RhetoricalStats,
    unique_lemmas: &mut std::collections::HashSet<String>,
) {
    match expr {
        Expr::Word(w) => {
            stats.total_words += 1;
            let analysis = analyze(&w.original);

            // Track vocabulary richness
            if !matches!(
                analysis.part_of_speech,
                PartOfSpeech::Particle | PartOfSpeech::Article
            ) {
                unique_lemmas.insert(analysis.lemma.to_string());
            }

            if analysis.part_of_speech == PartOfSpeech::Verb {
                stats.total_verbs += 1;

                if let Some(tense) = analysis.tense {
                    *stats.tense_counts.entry(tense).or_insert(0) += 1;
                }
                if let Some(mood) = analysis.mood {
                    *stats.mood_counts.entry(mood).or_insert(0) += 1;
                }
                if let Some(voice) = analysis.voice {
                    *stats.voice_counts.entry(voice).or_insert(0) += 1;
                }
            }
        }
        Expr::Call { verb, arguments } => {
            // Analyze the verb itself
            stats.total_words += 1;
            let analysis = analyze(&verb.original);

            if !matches!(
                analysis.part_of_speech,
                PartOfSpeech::Particle | PartOfSpeech::Article
            ) {
                unique_lemmas.insert(analysis.lemma.to_string());
            }

            if analysis.part_of_speech == PartOfSpeech::Verb {
                stats.total_verbs += 1;
                if let Some(tense) = analysis.tense {
                    *stats.tense_counts.entry(tense).or_insert(0) += 1;
                }
                if let Some(mood) = analysis.mood {
                    *stats.mood_counts.entry(mood).or_insert(0) += 1;
                }
                if let Some(voice) = analysis.voice {
                    *stats.voice_counts.entry(voice).or_insert(0) += 1;
                }
            }

            // Recurse into arguments
            for arg in arguments {
                analyze_expression(arg, stats, unique_lemmas);
            }
        }
        Expr::Phrase(exprs) => {
            for e in exprs {
                analyze_expression(e, stats, unique_lemmas);
            }
        }
        Expr::ArrayLiteral(exprs) => {
            for e in exprs {
                analyze_expression(e, stats, unique_lemmas);
            }
        }
        Expr::IndexAccess { array, index } => {
            analyze_expression(array, stats, unique_lemmas);
            analyze_expression(index, stats, unique_lemmas);
        }
        Expr::PropertyAccess { owner, property } => {
            analyze_expression(owner, stats, unique_lemmas);
            analyze_expression(property, stats, unique_lemmas);
        }
        Expr::BinOp { left, right, .. } => {
            analyze_expression(left, stats, unique_lemmas);
            analyze_expression(right, stats, unique_lemmas);
        }
        Expr::UnaryOp { operand, .. } => {
            analyze_expression(operand, stats, unique_lemmas);
        }
        Expr::Binding { name, value } => {
            // Analyze the value
            analyze_expression(value, stats, unique_lemmas);

            // Name is also a word
            stats.total_words += 1;
            unique_lemmas.insert(name.normalized.to_string());
        }
        Expr::Block(stmts) => {
            for s in stmts {
                analyze_statement(s, stats, unique_lemmas);
            }
        }
        Expr::Lambda { verb_lemma, .. } => {
            // Lambda is formed from a participle, which acts as a verb
            stats.total_words += 1;
            // The verb_lemma itself is the lemma, but we want to analyze the original form
            // Wait, we don't have the original participle form here easily in Expr::Lambda unless we stored it?
            // ast::Expr::Lambda has `verb_lemma: String`.
            // We can try to analyze the lemma itself, but it's already a lemma.
            // Let's just count it as a verb.
            unique_lemmas.insert(verb_lemma.clone());
            stats.total_verbs += 1;

            // We can't easily guess tense/mood from just the lemma without the original participle ending.
            // But we know it's a participle.
            // Let's skip specific tense/mood counting for now unless we change AST.
        }
        _ => {} // Literals, etc.
    }
}

fn calculate_derived_stats(stats: &mut RhetoricalStats, unique_lemma_count: usize) {
    if stats.total_words > 0 {
        // Eloquence = (Unique Lemmas / Total Words) * 100
        // A measure of "non-repetitiveness"
        stats.eloquence_score = (unique_lemma_count as f32 / stats.total_words as f32) * 100.0;
    }

    // Determine Tone
    let imperative_count = *stats.mood_counts.get(&Mood::Imperative).unwrap_or(&0);
    let indicative_count = *stats.mood_counts.get(&Mood::Indicative).unwrap_or(&0);

    if stats.total_verbs == 0 {
        stats.tone = "Silent".to_string();
    } else if imperative_count > indicative_count {
        stats.tone = "Commanding (High Imperative)".to_string();
    } else {
        stats.tone = "Descriptive (High Indicative)".to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_imperative_tone() {
        let code = "«χαῖρε» λέγε.";
        let program = parse(code).expect("Failed to parse");
        let stats = analyze_rhetoric(&program);

        // "λέγε" is imperative
        assert!(stats.total_verbs >= 1, "Expected at least 1 verb");
        let imperative = *stats.mood_counts.get(&Mood::Imperative).unwrap_or(&0);
        assert!(imperative >= 1, "Expected imperative mood");
        assert!(stats.tone.contains("Commanding"));
    }

    #[test]
    fn test_indicative_tone() {
        let code = "ὁ ἄνθρωπος λέγει."; // "The man speaks."
        let program = parse(code).expect("Failed to parse");
        let stats = analyze_rhetoric(&program);

        // "λέγει" is indicative
        assert!(stats.total_verbs >= 1, "Expected at least 1 verb");
        let indicative = *stats.mood_counts.get(&Mood::Indicative).unwrap_or(&0);
        assert!(indicative >= 1, "Expected indicative mood");
        assert!(stats.tone.contains("Descriptive"));
    }

    #[test]
    fn test_eloquence_score() {
        // High variety: 3 distinct words (excluding articles)
        let code_high = "ὁ ἄνθρωπος τὸν λόγον λέγει.";
        let prog_high = parse(code_high).expect("Failed to parse high");
        let stats_high = analyze_rhetoric(&prog_high);

        // Low variety: repeated words
        let code_low = "λέγε λέγε λέγε.";
        let prog_low = parse(code_low).expect("Failed to parse low");
        let stats_low = analyze_rhetoric(&prog_low);

        assert!(
            stats_high.eloquence_score > stats_low.eloquence_score,
            "High variety should have higher eloquence score. High: {}, Low: {}",
            stats_high.eloquence_score,
            stats_low.eloquence_score
        );
    }
}
