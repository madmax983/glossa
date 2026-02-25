            // If we have a subject, create a property access (use normalized original, not lemma)
            if let Some(ref subj) = self.state.subject {
                if self.state.property_accesses.len() >= MAX_PROPERTY_ACCESSES {
                    return Err(AssemblyError::LimitExceeded {
                        resource: "Property Accesses".to_string(),
                        max: MAX_PROPERTY_ACCESSES,
                    });
                }
                // OPTIMIZATION: Use stored normalized form
                self.state
                    .property_accesses
                    .push((subj.normalized.to_string(), "len".to_string()));
                self.state.subject = None; // Consume the subject
                return Ok(true);
            }
        }

        // Ordinal adjectives
        if crate::morphology::lexicon::is_ordinal(normalized) {
            // If we have a subject, create an index access with the ordinal index
            if let Some(ref subj) = self.state.subject
                && let Some(index) = crate::morphology::lexicon::ordinal_to_index(normalized)
            {
                if self.state.index_accesses.len() >= MAX_INDEX_ACCESSES {
                    return Err(AssemblyError::LimitExceeded {
                        resource: "Index Accesses".to_string(),
                        max: MAX_INDEX_ACCESSES,
                    });
                }
                // Create array and index expressions (use normalized original, not lemma)
                // OPTIMIZATION: Use stored normalized form
                let array = Expr::Word(Word {
                    original: subj.original.clone(),
                    normalized: subj.normalized.clone(),
                });
                let index_expr = Expr::NumberLiteral(index);

                self.state.index_accesses.push((array, index_expr));
                self.state.subject = None; // Consume the subject
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check subject-verb agreement
    fn check_agreement(
        &self,
        subject: &Constituent,
        verb: &VerbConstituent,
    ) -> Result<(), AssemblyError> {
        if let (Some(verb_person), Some(verb_number), Some(subj_number)) =
            (verb.person, verb.number, subject.number)
        {
            // Determine subject person (default to 3rd for nouns if not specified)
            let subj_person = subject.person.unwrap_or(Person::Third);

            // Check person agreement
            // Exception: Allow Imperative verbs to disagree (e.g. "User, print!" uses 2nd person verb with 3rd person subject)
            let is_imperative = verb.mood == Some(Mood::Imperative);
            if !is_imperative && subj_person != verb_person {
                return Err(AssemblyError::SubjectVerbDisagreement {
                    subject: (Some(subj_person), Some(subj_number)),
                    verb: (Some(verb_person), Some(verb_number)),
                });
            }

            // Special rule: Neuter plural nouns take singular verbs in Greek!
            let is_neuter_plural =
                subject.gender == Some(Gender::Neuter) && subj_number == Number::Plural;

            if !is_neuter_plural && subj_number != verb_number {
                return Err(AssemblyError::SubjectVerbDisagreement {
                    subject: (Some(subj_person), Some(subj_number)),
                    verb: (Some(verb_person), Some(verb_number)),
                });
            }
        }
        Ok(())
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::{Tense, Voice, analyze};

    #[test]
    fn test_operator_detection() {
        // μεῖζον should be detected as > operator
        let mut asm = Assembler::new();

        // Feed comparison adjective
        let meizon = analyze("μειζον");
        asm.feed(&meizon, "μεῖζον").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(
            !stmt.operators.is_empty(),
            "Expected operator to be captured"
        );
        assert_eq!(stmt.operators[0], BinaryOp::Gt);
    }

    #[test]
    fn test_boolean_or_detection() {
        // ἤ should be detected as || operator
        let mut asm = Assembler::new();

        // Feed boolean particle
        let or_particle = analyze("η");
        asm.feed(&or_particle, "ἤ").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(
            !stmt.operators.is_empty(),
            "Expected operator to be captured, got: {:?}",
            stmt
        );
        assert_eq!(stmt.operators[0], BinaryOp::Or);
    }

    #[test]
    fn test_full_boolean_or_expression() {
        // ἀληθές ἤ ψεῦδος λέγε - simulate the full expression
        let mut asm = Assembler::new();

        // Feed true (boolean literal - handled by parser, goes to feed_boolean)
        asm.feed_boolean(true).unwrap();

        // Feed ἤ (OR operator)
        let or_particle = analyze("η");
        asm.feed(&or_particle, "ἤ").unwrap();

        // Feed false (boolean literal)
        asm.feed_boolean(false).unwrap();

        // Feed λέγε (print verb)
        let verb = analyze("λεγε");
        asm.feed(&verb, "λέγε").unwrap();

        let stmt = asm.finalize().unwrap();
        assert_eq!(
            stmt.literals.len(),
            2,
            "Expected 2 literals, got: {:?}",
            stmt.literals
        );
        assert_eq!(
            stmt.operators.len(),
            1,
            "Expected 1 operator, got: {:?}",
            stmt.operators
        );
        assert_eq!(stmt.operators[0], BinaryOp::Or);
    }

    #[test]
    fn test_simple_sov() {
        // ὁ ἄνθρωπος τὸν λόγον λέγει (The man says the word)
        let mut asm = Assembler::new();

        // Feed subject (nominative)
        let subj = analyze("ανθρωπος");
        asm.feed(&subj, "ἄνθρωπος").unwrap();

        // Feed object (accusative)
        let obj = analyze("λογον");
        asm.feed(&obj, "λόγον").unwrap();

        // Feed verb
        let verb = analyze("λεγει");
        asm.feed(&verb, "λέγει").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(stmt.subject.is_some());
        assert!(stmt.object.is_some());
        assert!(stmt.verb.is_some());
    }

    #[test]
    fn test_vso_same_result() {
        // λέγει τὸν λόγον ὁ ἄνθρωπος (VSO - same meaning)
        let mut asm = Assembler::new();

        // Feed verb first
        let verb = analyze("λεγει");
        asm.feed(&verb, "λέγει").unwrap();

        // Feed object
        let obj = analyze("λογον");
        asm.feed(&obj, "λόγον").unwrap();

        // Feed subject
        let subj = analyze("ανθρωπος");
        asm.feed(&subj, "ἄνθρωπος").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(stmt.subject.is_some());
        assert!(stmt.object.is_some());
        assert!(stmt.verb.is_some());
    }

    #[test]
    fn test_multiple_nominatives() {
        // Multiple nominatives are now allowed for function call patterns
        let mut asm = Assembler::new();

        let subj1 = analyze("ανθρωπος");
        asm.feed(&subj1, "ἄνθρωπος").unwrap();

        let subj2 = analyze("θεος");
        asm.feed(&subj2, "θεός").unwrap(); // Should succeed now

        let stmt = asm.finalize().unwrap();
        assert!(stmt.subject.is_some());
        assert_eq!(stmt.nominatives.len(), 1); // Second nominative in the list
    }

    #[test]
    fn test_double_verb_error() {
        let mut asm = Assembler::new();

        let verb1 = analyze("λεγει");
        asm.feed(&verb1, "λέγει").unwrap();

        let verb2 = analyze("γραφει");
        let result = asm.feed(&verb2, "γράφει");

        assert!(matches!(result, Err(AssemblyError::DoubleVerb)));
    }

    #[test]
    fn test_literals() {
        let mut asm = Assembler::new();

        asm.feed_string("χαῖρε κόσμε".to_string()).unwrap();

        let verb = analyze("λεγε");
        asm.feed(&verb, "λέγε").unwrap();

        let stmt = asm.finalize().unwrap();
        assert_eq!(stmt.literals.len(), 1);
        assert!(matches!(&stmt.literals[0], Literal::String(s) if s == "χαῖρε κόσμε"));
    }

    #[test]
    fn test_genitive_possession() {
        let mut asm = Assembler::new();

        // χρήστου ὄνομα (the name of the user)
        let genitive = analyze("χρηστου");
        asm.feed(&genitive, "χρήστου").unwrap();

        let nom = analyze("ονομα");
        asm.feed(&nom, "ὄνομα").unwrap();

        let stmt = asm.finalize().unwrap();
        assert_eq!(stmt.genitives.len(), 1);
        assert!(stmt.subject.is_some() || stmt.object.is_some());
    }

    #[test]
    fn test_dative_indirect_object() {
        let mut asm = Assembler::new();

        // τῷ ἀνθρώπῳ δίδωμι (I give to the man)
        let dat = analyze("ανθρωπω");
        asm.feed(&dat, "ἀνθρώπῳ").unwrap();

        let verb = analyze("διδωμι");
        asm.feed(&verb, "δίδωμι").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(stmt.indirect.is_some());
    }

    #[test]
    fn test_verb_constituent_has_voice() {
        let mut asm = Assembler::new();

        // γίγνεται - middle voice verb
        let verb = analyze("γιγνεται");
        asm.feed(&verb, "γίγνεται").unwrap();

        let stmt = asm.finalize().unwrap();
        assert!(stmt.verb.is_some());
        let verb_const = stmt.verb.unwrap();
        assert_eq!(verb_const.voice, Some(Voice::Middle));
    }

    #[test]
    fn test_subject_verb_person_agreement() {
        let mut asm = Assembler::new();

        // Feed subject "ἐγώ" (I) - First Person Singular
        // Manually construct analysis since "ego" might not be in the simple lexicon used in tests
        let ego_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("εγω"),
            part_of_speech: PartOfSpeech::Pronoun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::First), // KEY: First Person
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&ego_analysis, "ἐγώ").unwrap();

        // Feed verb "λέγει" (He says) - Third Person Singular
        let verb_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("λεγω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::Third), // KEY: Third Person
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 1.0,
        };

        // This should fail IMMEDIATELY during feed because we have strict agreement checks now
        let result = asm.feed(&verb_analysis, "λέγει");

        assert!(
            matches!(result, Err(AssemblyError::SubjectVerbDisagreement { .. })),
            "Expected immediate agreement failure"
        );
    }

    #[test]
    fn test_double_object_error() {
        let mut asm = Assembler::new();

        // First object: λόγον
        let obj1 = analyze("λόγον");
        asm.feed(&obj1, "λόγον").unwrap();

        // Second object: λόγον (again)
        let obj2 = analyze("λόγον");
        let result = asm.feed(&obj2, "λόγον");

        assert!(matches!(result, Err(AssemblyError::DoubleObject)));
    }

    #[test]
    fn test_neuter_plural_subject_singular_verb() {
        let mut asm = Assembler::new();

        // Subject: τὰ ζῷα (The animals) - Neuter Plural
        let subj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("ζωον"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Plural),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj, "ζῷα").unwrap();

        // Verb: τρέχει (runs) - Singular
        let verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("τρεχω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular), // Singular!
            gender: None,
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&verb, "τρέχει").unwrap();

        // Should succeed despite Plural Subject + Singular Verb
        let stmt = asm.finalize();
        assert!(
            stmt.is_ok(),
            "Neuter plural subject should agree with singular verb, got {:?}",
            stmt.err()
        );
    }

    #[test]
    fn test_imperative_mismatch() {
        let mut asm = Assembler::new();

        // Subject: "User" (3rd person)
        let subj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("User"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Masculine),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj, "User").unwrap();

        // Verb: "Print!" (Imperative, 2nd person)
        let verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("print"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::Second), // 2nd person
            tense: None,
            mood: Some(Mood::Imperative), // Imperative
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&verb, "Print").unwrap();

        // Should succeed
        let stmt = asm.finalize();
        assert!(
            stmt.is_ok(),
            "Imperative verb should allow person mismatch, got {:?}",
            stmt.err()
        );
    }

    #[test]
    fn test_gender_mismatch_ignored() {
        // This test verifies that Gender Mismatch is CURRENTLY IGNORED.
        let mut asm = Assembler::new();

        // Adjective: καλός (Masculine)
        let adj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("καλος"),
            part_of_speech: PartOfSpeech::Adjective,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Masculine),
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&adj, "καλός").unwrap();

        // Noun: γυνή (Feminine)
        let noun = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("γυνη"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Feminine),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&noun, "γυνή").unwrap();

        // Verb (to complete the sentence)
        let verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("λεγω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&verb, "λέγει").unwrap();

        let stmt = asm.finalize();
        // Currently expecting OK because the check is missing
        assert!(stmt.is_ok(), "Gender mismatch is currently ignored");
    }

    #[test]
    fn test_split_method_generation() {
        let mut asm = Assembler::new();

        // 1. Subject: "text"
        let subj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("text"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj, "text").unwrap();

        // 2. Delimiter Preposition: "κατά"
        let marker_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("κατα"),
            part_of_speech: PartOfSpeech::Preposition,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&marker_analysis, "κατά").unwrap();

        // 3. Delimiter Literal: ","
        asm.feed_string(",".to_string()).unwrap();

        // 4. Split Verb: "σχίζεται" (is split)
        let split_verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("σχιζω"), // assuming lemma for split verb
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: None,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&split_verb, "σχίζεται").unwrap();

        let stmt = asm.finalize().unwrap();

        // Check if property access was created
        assert!(
            !stmt.property_accesses.is_empty(),
            "Should generate property access for split"
        );
        assert_eq!(stmt.property_accesses[0].1, "split");

        // Check if string method info was captured
        assert_eq!(
            stmt.string_method,
            Some(("split".to_string(), ",".to_string()))
        );
    }

    #[test]
    fn test_immediate_agreement_failure_vso() {
        let mut asm = Assembler::new();

        // Feed verb: "I see" (1st Person Singular)
        let verb_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("βλεπω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::First),
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 1.0,
        };
        asm.feed(&verb_analysis, "βλέπω").unwrap();

        // Feed subject: "The gift" (3rd Person Singular)
        let subj_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("δωρον"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };

        // Should fail IMMEDIATELY because "I see" (1st) != "gift" (3rd)
        let result = asm.feed(&subj_analysis, "δῶρον");
        assert!(
            matches!(result, Err(AssemblyError::SubjectVerbDisagreement { .. })),
            "Expected immediate agreement failure for VSO"
        );
    }

    #[test]
    fn test_immediate_agreement_failure_svo() {
        let mut asm = Assembler::new();

        // Feed subject: "The gift" (3rd Person Singular)
        let subj_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("δωρον"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Singular),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj_analysis, "δῶρον").unwrap();

        // Feed verb: "I see" (1st Person Singular)
        let verb_analysis = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("βλεπω"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::First),
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 1.0,
        };

        // Should fail IMMEDIATELY because "gift" (3rd) != "I see" (1st)
        let result = asm.feed(&verb_analysis, "βλέπω");
        assert!(
            matches!(result, Err(AssemblyError::SubjectVerbDisagreement { .. })),
            "Expected immediate agreement failure for SVO"
        );
    }

    fn make_analysis(
        lemma: &str,
        pos: PartOfSpeech,
        case: Option<Case>,
        number: Option<Number>,
    ) -> MorphAnalysis {
        MorphAnalysis {
            lemma: std::borrow::Cow::Owned(lemma.to_string()),
            part_of_speech: pos,
            case,
            number,
            gender: None,
            person: None,
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        }
    }

    #[test]
    fn test_max_literals_exceeded() {
        let mut asm = Assembler::new();
        for i in 0..MAX_LITERALS {
            asm.feed_number(i as i64).unwrap();
        }
        let result = asm.feed_number(0);
        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Literals" && max == MAX_LITERALS)
        );
    }

    #[test]
    fn test_max_nominatives_exceeded() {
        let mut asm = Assembler::new();
        let subj = make_analysis(
            "subject",
            PartOfSpeech::Noun,
            Some(Case::Nominative),
            Some(Number::Singular),
        );
        asm.feed(&subj, "subject").unwrap();

        for i in 0..MAX_NOMINATIVES {
            let nom = make_analysis(
                &format!("nom_{}", i),
                PartOfSpeech::Noun,
                Some(Case::Nominative),
                Some(Number::Singular),
            );
            asm.feed(&nom, &format!("nom_{}", i)).unwrap();
        }

        let nom = make_analysis(
            "overflow",
            PartOfSpeech::Noun,
            Some(Case::Nominative),
            Some(Number::Singular),
        );
        let result = asm.feed(&nom, "overflow");

        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Nominatives" && max == MAX_NOMINATIVES)
        );
    }

    #[test]
    fn test_max_adjectives_exceeded() {
        let mut asm = Assembler::new();
        for i in 0..MAX_ADJECTIVES {
            let adj = make_analysis(
                &format!("adj_{}", i),
                PartOfSpeech::Adjective,
                Some(Case::Nominative),
                Some(Number::Singular),
            );
            asm.feed(&adj, &format!("adj_{}", i)).unwrap();
        }

        let adj = make_analysis(
            "overflow",
            PartOfSpeech::Adjective,
            Some(Case::Nominative),
            Some(Number::Singular),
        );
        let result = asm.feed(&adj, "overflow");

        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Adjectives" && max == MAX_ADJECTIVES)
        );
    }

    #[test]
    fn test_max_operators_exceeded() {
        let mut asm = Assembler::new();
        let op_analysis = make_analysis("και", PartOfSpeech::Conjunction, None, None);
        for _ in 0..MAX_OPERATORS {
            asm.feed(&op_analysis, "καί").unwrap();
        }
        let result = asm.feed(&op_analysis, "καί");

        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Operators" && max == MAX_OPERATORS)
        );
    }

    #[test]
    fn test_max_genitives_exceeded() {
        let mut asm = Assembler::new();
        for i in 0..MAX_GENITIVES {
            let genitive = make_analysis(
                &format!("gen_{}", i),
                PartOfSpeech::Noun,
                Some(Case::Genitive),
                Some(Number::Singular),
            );
            asm.feed(&genitive, &format!("gen_{}", i)).unwrap();
        }

        let genitive = make_analysis(
            "overflow",
            PartOfSpeech::Noun,
            Some(Case::Genitive),
            Some(Number::Singular),
        );
        let result = asm.feed(&genitive, "overflow");

        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Genitives" && max == MAX_GENITIVES)
        );
    }

    #[test]
    fn test_silent_swallowing_of_unknown_case() {
        let mut asm = Assembler::new();
        let obj = make_analysis(
            "object",
            PartOfSpeech::Noun,
            Some(Case::Accusative),
            Some(Number::Singular),
        );
        asm.feed(&obj, "object").unwrap();

        let unknown = make_analysis("unknown", PartOfSpeech::Noun, None, Some(Number::Singular));
        let result = asm.feed(&unknown, "unknown");

        assert!(
            matches!(result, Err(AssemblyError::DoubleObject)),
            "Expected DoubleObject error for unknown case when object slot is full, got {:?}",
            result
        );
    }

    #[test]
    fn test_neuter_plural_subject_first_person_verb() {
        let mut asm = Assembler::new();
        let subj = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("dwron"),
            part_of_speech: PartOfSpeech::Noun,
            case: Some(Case::Nominative),
            number: Some(Number::Plural),
            gender: Some(Gender::Neuter),
            person: Some(Person::Third),
            tense: None,
            mood: None,
            voice: None,
            confidence: 1.0,
        };
        asm.feed(&subj, "δῶρα").unwrap();

        let verb = MorphAnalysis {
            lemma: std::borrow::Cow::Borrowed("blepw"),
            part_of_speech: PartOfSpeech::Verb,
            case: None,
            number: Some(Number::Singular),
            gender: None,
            person: Some(Person::First),
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(Voice::Active),
            confidence: 1.0,
        };

        let result = asm.feed(&verb, "βλέπω");
        assert!(
            matches!(result, Err(AssemblyError::SubjectVerbDisagreement { .. })),
            "Neuter plural subject (3rd) should NOT agree with 1st person verb, got {:?}",
            result
        );
    }

    #[test]
    fn test_disambiguation_en_vs_hen() {
        let mut asm = Assembler::new();
        let analysis = make_analysis("εν", PartOfSpeech::Preposition, None, None);

        asm.feed(&analysis, "ἐν").unwrap();
        let stmt = asm.finalize().unwrap();
        assert!(stmt.has_containment_preposition);

        let mut asm = Assembler::new();
        let hen = "ἕν";
        asm.feed(&analysis, hen).unwrap();
        let stmt = asm.finalize().unwrap();
        assert!(
            !stmt.has_containment_preposition,
            "Should not detect containment preposition for 'one' (hen)"
        );
    }

    #[test]
    fn test_max_arrays_exceeded() {
        let mut asm = Assembler::new();
        for _ in 0..MAX_ARRAYS {
            asm.feed_array(vec![]).unwrap();
        }
        let result = asm.feed_array(vec![]);
        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Arrays" && max == MAX_ARRAYS)
        );
    }

    #[test]
    fn test_max_index_accesses_exceeded() {
        let mut asm = Assembler::new();
        let array = Expr::NumberLiteral(0); // Dummy expression
        let index = Expr::NumberLiteral(0);

        for _ in 0..MAX_INDEX_ACCESSES {
            asm.feed_index_access(array.clone(), index.clone()).unwrap();
        }
        let result = asm.feed_index_access(array, index);
        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Index Accesses" && max == MAX_INDEX_ACCESSES)
        );
    }

    #[test]
    fn test_max_nested_phrases_exceeded() {
        let mut asm = Assembler::new();
        for _ in 0..MAX_NESTED_PHRASES {
            asm.feed_nested_phrase(vec![]).unwrap();
        }
        let result = asm.feed_nested_phrase(vec![]);
        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Nested Phrases" && max == MAX_NESTED_PHRASES)
        );
    }

    #[test]
    fn test_max_participles_exceeded() {
        let mut asm = Assembler::new();
        let analysis = crate::morphology::ParticipleAnalysis {
            stem: "stem".into(),
            tense: crate::morphology::Tense::Present,
            voice: crate::morphology::Voice::Active,
            case: crate::morphology::Case::Nominative,
            gender: crate::morphology::Gender::Masculine,
            number: crate::morphology::Number::Singular,
            confidence: 1.0,
        };

        for i in 0..MAX_PARTICIPLES {
            asm.feed_participle(&analysis, &format!("part_{}", i))
                .unwrap();
        }
        let result = asm.feed_participle(&analysis, "overflow");
        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Participles" && max == MAX_PARTICIPLES)
        );
    }

    #[test]
    fn test_max_unwraps_exceeded() {
        let mut asm = Assembler::new();
        let expr = Expr::NumberLiteral(0);
        for _ in 0..MAX_UNWRAPS {
            asm.feed_unwrap(expr.clone()).unwrap();
        }
        let result = asm.feed_unwrap(expr);
        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Unwraps" && max == MAX_UNWRAPS)
        );
    }

    #[test]
    fn test_max_blocks_exceeded() {
        let mut asm = Assembler::new();
        for _ in 0..MAX_BLOCKS {
            asm.feed_block(vec![]).unwrap();
        }
        let result = asm.feed_block(vec![]);
        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Blocks" && max == MAX_BLOCKS)
        );
    }

    #[test]
    fn test_max_property_accesses_exceeded() {
        let mut asm = Assembler::new();

        // Fill up to the limit
        for _ in 0..MAX_PROPERTY_ACCESSES {
            // Replenish subject because check_special_properties consumes it
            let subj = make_analysis(
                "subject",
                PartOfSpeech::Noun,
                Some(Case::Nominative),
                Some(Number::Singular),
            );
            asm.feed(&subj, "subject").unwrap();

            // Feed property "μῆκος" (length)
            let prop = make_analysis("μηκος", PartOfSpeech::Noun, None, None);
            asm.feed_with_normalized(&prop, "μῆκος", "μηκος").unwrap();
        }

        // Try one more time to break it
        let subj = make_analysis(
            "subject",
            PartOfSpeech::Noun,
            Some(Case::Nominative),
            Some(Number::Singular),
        );
        asm.feed(&subj, "subject").unwrap();

        let prop = make_analysis("μηκος", PartOfSpeech::Noun, None, None);
        let result = asm.feed_with_normalized(&prop, "μῆκος", "μηκος");

        assert!(
            matches!(result, Err(AssemblyError::LimitExceeded { ref resource, max }) if resource == "Property Accesses" && max == MAX_PROPERTY_ACCESSES)
        );
    }

    #[test]
    fn test_unknown_case_becomes_object_when_slot_empty() {
        let mut asm = Assembler::new();

        // Feed unknown word
        // PartOfSpeech::Noun but case: None
        let unknown = make_analysis("unknown", PartOfSpeech::Noun, None, Some(Number::Singular));
        asm.feed(&unknown, "unknown").unwrap();

        let stmt = asm.finalize().unwrap();

        // Assert it was captured as object
        assert!(
            stmt.object.is_some(),
            "Unknown word should have been captured as object"
        );
        assert_eq!(stmt.object.unwrap().original, "unknown");
    }
}
