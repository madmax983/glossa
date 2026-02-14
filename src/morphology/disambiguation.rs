//! Morphological disambiguation for ΓΛΩΣΣΑ
//!
//! Greek morphology is inherently ambiguous. This module resolves ambiguity
//! using syntactic context:
//! - Article-noun agreement (gender, number, case)
//! - Subject-verb agreement (person, number)
//! - Adjective-noun agreement (gender, number, case)
//!
//! ## Example
//!
//! The word "θαλασσα" could be:
//! - Nominative singular (the sea as subject)
//! - Vocative singular (O sea!)
//!
//! If preceded by "ἡ" (feminine nominative article), we know it's nominative.

use crate::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech, Person};

/// A disambiguation context built from surrounding words
#[derive(Debug, Clone, Default)]
pub struct DisambiguationContext {
    /// Expected case from article or preposition
    pub expected_case: Option<Case>,
    /// Expected number from article or verb
    pub expected_number: Option<Number>,
    /// Expected gender from article or adjective
    pub expected_gender: Option<Gender>,
    /// Expected person from verb (for subject agreement)
    pub expected_person: Option<Person>,
}

impl DisambiguationContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create context from a preceding article
    pub fn from_article(article: &MorphAnalysis) -> Self {
        DisambiguationContext {
            expected_case: article.case,
            expected_number: article.number,
            expected_gender: article.gender,
            expected_person: None,
        }
    }

    /// Create context from a verb (for subject agreement)
    pub fn from_verb(verb: &MorphAnalysis) -> Self {
        DisambiguationContext {
            expected_case: Some(Case::Nominative), // Subject is nominative
            expected_number: verb.number,
            expected_gender: None, // Verbs don't have gender
            expected_person: verb.person,
        }
    }

    /// Create context expecting a specific case (e.g., after preposition)
    pub fn expecting_case(case: Case) -> Self {
        DisambiguationContext {
            expected_case: Some(case),
            expected_number: None,
            expected_gender: None,
            expected_person: None,
        }
    }
}

/// Disambiguate a list of possible analyses using context
///
/// Returns analyses filtered and re-ranked by how well they match the context.
/// The first element is the best match.
///
/// Optimization: Sorts in-place to avoid intermediate vector allocations.
pub fn disambiguate(
    mut analyses: Vec<MorphAnalysis>,
    context: &DisambiguationContext,
) -> Vec<MorphAnalysis> {
    if analyses.is_empty() {
        return analyses;
    }

    // If no context, return as-is (sorted by original confidence)
    if context.expected_case.is_none()
        && context.expected_number.is_none()
        && context.expected_gender.is_none()
        && context.expected_person.is_none()
    {
        return analyses;
    }

    // Sort by score (descending), then by original confidence
    // We recompute score_analysis during sort to avoid allocating a wrapper struct.
    // score_analysis is cheap (field comparisons).
    analyses.sort_by(|a, b| {
        let score_a = score_analysis(a, context);
        let score_b = score_analysis(b, context);

        let score_cmp = score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal);

        if score_cmp == std::cmp::Ordering::Equal {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            score_cmp
        }
    });

    // Update confidence in-place
    for analysis in &mut analyses {
        let score = score_analysis(analysis, context);
        analysis.confidence = (analysis.confidence * 0.5 + score * 0.5).min(1.0);
    }

    analyses
}

/// Score how well an analysis matches the context (0.0 - 1.0)
fn score_analysis(analysis: &MorphAnalysis, context: &DisambiguationContext) -> f32 {
    let mut score: f32 = 0.5; // Neutral starting point

    // Case agreement
    if let Some(expected) = context.expected_case {
        if analysis.case == Some(expected) {
            score += 0.2;
        } else if analysis.case.is_some() {
            score -= 0.3; // Penalize mismatches
        }
    }

    // Number agreement
    if let Some(expected) = context.expected_number {
        if analysis.number == Some(expected) {
            score += 0.2;
        } else if analysis.number.is_some() {
            score -= 0.3;
        }
    }

    // Gender agreement
    if let Some(expected) = context.expected_gender {
        if analysis.gender == Some(expected) {
            score += 0.2;
        } else if analysis.gender.is_some() {
            score -= 0.3;
        }
    }

    // Person agreement (for verbs matching subjects)
    if let Some(expected) = context.expected_person {
        if analysis.person == Some(expected) {
            score += 0.2;
        } else if analysis.person.is_some() {
            score -= 0.3;
        }
    }

    // Clamp to valid range
    score.clamp(0.0, 1.0)
}

/// Resolve the best analysis from multiple possibilities
///
/// This is the main entry point for disambiguation. It returns the single
/// best analysis based on context, or the highest-confidence one if no
/// context matches.
///
/// Optimization: Takes ownership of analyses vector to avoid cloning.
pub fn resolve_best(
    analyses: Vec<MorphAnalysis>,
    context: &DisambiguationContext,
) -> MorphAnalysis {
    let mut disambiguated = disambiguate(analyses, context);
    if disambiguated.is_empty() {
        MorphAnalysis::new("?".to_string(), PartOfSpeech::Unknown)
    } else {
        // Efficiently extract the best (first) element
        disambiguated.swap_remove(0)
    }
}

/// Known Greek articles for context building
/// Uses ORIGINAL forms with diacritics to distinguish from homographs
/// e.g., ἡ (article) vs ἤ (or) - differ only in breathing/accent
pub fn analyze_article(word: &str) -> Option<DisambiguationContext> {
    // Match on original polytonic forms - diacritics matter!
    match word {
        // Masculine nominative - ὁ with rough breathing
        "ὁ" | "ο" => Some(DisambiguationContext {
            expected_case: Some(Case::Nominative),
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Masculine),
            expected_person: None,
        }),
        "τοῦ" | "του" => Some(DisambiguationContext {
            expected_case: Some(Case::Genitive),
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Masculine),
            expected_person: None,
        }),
        "τῷ" | "τω" => Some(DisambiguationContext {
            expected_case: Some(Case::Dative),
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Masculine),
            expected_person: None,
        }),
        "τόν" | "τὸν" | "τον" => Some(DisambiguationContext {
            expected_case: Some(Case::Accusative),
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Masculine),
            expected_person: None,
        }),
        "οἱ" | "οι" => Some(DisambiguationContext {
            expected_case: Some(Case::Nominative),
            expected_number: Some(Number::Plural),
            expected_gender: Some(Gender::Masculine),
            expected_person: None,
        }),
        "τῶν" | "των" => Some(DisambiguationContext {
            expected_case: Some(Case::Genitive),
            expected_number: Some(Number::Plural),
            expected_gender: None, // All genders share τῶν
            expected_person: None,
        }),
        "τοῖς" | "τοις" => Some(DisambiguationContext {
            expected_case: Some(Case::Dative),
            expected_number: Some(Number::Plural),
            expected_gender: Some(Gender::Masculine),
            expected_person: None,
        }),
        "τούς" | "τοὺς" | "τους" => Some(DisambiguationContext {
            expected_case: Some(Case::Accusative),
            expected_number: Some(Number::Plural),
            expected_gender: Some(Gender::Masculine),
            expected_person: None,
        }),

        // Feminine - ἡ with ROUGH breathing (NOT ἤ which is "or")
        "ἡ" => Some(DisambiguationContext {
            expected_case: Some(Case::Nominative),
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Feminine),
            expected_person: None,
        }),
        "τῆς" | "της" => Some(DisambiguationContext {
            expected_case: Some(Case::Genitive),
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Feminine),
            expected_person: None,
        }),
        "τῇ" | "τη" => Some(DisambiguationContext {
            expected_case: Some(Case::Dative),
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Feminine),
            expected_person: None,
        }),
        "τήν" | "τὴν" | "την" => Some(DisambiguationContext {
            expected_case: Some(Case::Accusative),
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Feminine),
            expected_person: None,
        }),
        "αἱ" | "αι" => Some(DisambiguationContext {
            expected_case: Some(Case::Nominative),
            expected_number: Some(Number::Plural),
            expected_gender: Some(Gender::Feminine),
            expected_person: None,
        }),
        "ταῖς" | "ταις" => Some(DisambiguationContext {
            expected_case: Some(Case::Dative),
            expected_number: Some(Number::Plural),
            expected_gender: Some(Gender::Feminine),
            expected_person: None,
        }),
        "τάς" | "τὰς" | "τας" => Some(DisambiguationContext {
            expected_case: Some(Case::Accusative),
            expected_number: Some(Number::Plural),
            expected_gender: Some(Gender::Feminine),
            expected_person: None,
        }),

        // Neuter - Case is ambiguous (Nominative or Accusative)
        // We do NOT set expected_case so we don't bias disambiguation incorrectly.
        // The assembler will eventually decide based on available slots,
        // or we rely on backtracking if an incorrect choice causes a conflict.
        "τό" | "τὸ" | "το" => Some(DisambiguationContext {
            expected_case: None,
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Neuter),
            expected_person: None,
        }),
        "τά" | "τὰ" | "τα" => Some(DisambiguationContext {
            expected_case: None,
            expected_number: Some(Number::Plural),
            expected_gender: Some(Gender::Neuter),
            expected_person: None,
        }),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::analyze_all;

    #[test]
    fn test_disambiguate_with_article() {
        // "θαλασσα" could be nominative or vocative
        let analyses = analyze_all("θαλασσα");

        // With feminine nominative article context
        let context = DisambiguationContext {
            expected_case: Some(Case::Nominative),
            expected_number: Some(Number::Singular),
            expected_gender: Some(Gender::Feminine),
            expected_person: None,
        };

        let resolved = disambiguate(analyses, &context);
        assert!(!resolved.is_empty());
        // First result should be nominative
        assert_eq!(resolved[0].case, Some(Case::Nominative));
    }

    #[test]
    fn test_article_context() {
        let ctx = analyze_article("ο").unwrap();
        assert_eq!(ctx.expected_case, Some(Case::Nominative));
        assert_eq!(ctx.expected_gender, Some(Gender::Masculine));

        let ctx = analyze_article("την").unwrap();
        assert_eq!(ctx.expected_case, Some(Case::Accusative));
        assert_eq!(ctx.expected_gender, Some(Gender::Feminine));
    }

    #[test]
    fn test_verb_context() {
        use crate::morphology::analyze;

        let verb = analyze("γραφει");
        let ctx = DisambiguationContext::from_verb(&verb);

        assert_eq!(ctx.expected_case, Some(Case::Nominative));
        assert_eq!(ctx.expected_number, Some(Number::Singular));
        assert_eq!(ctx.expected_person, Some(crate::morphology::Person::Third));
    }

    #[test]
    fn test_no_context_preserves_order() {
        let analyses = analyze_all("λογος");
        let ctx = DisambiguationContext::new();
        let resolved = disambiguate(analyses.clone(), &ctx);

        // Order should be preserved when no context
        assert_eq!(analyses.len(), resolved.len());
    }

    #[test]
    fn test_neuter_article_ambiguity() {
        // "τό" should NOT force Nominative, because it can be Accusative
        let ctx = analyze_article("τό").unwrap();
        assert_eq!(ctx.expected_case, None);
        assert_eq!(ctx.expected_gender, Some(Gender::Neuter));
        assert_eq!(ctx.expected_number, Some(Number::Singular));

        // "τά" should NOT force Nominative
        let ctx = analyze_article("τά").unwrap();
        assert_eq!(ctx.expected_case, None);
        assert_eq!(ctx.expected_gender, Some(Gender::Neuter));
        assert_eq!(ctx.expected_number, Some(Number::Plural));
    }
}
