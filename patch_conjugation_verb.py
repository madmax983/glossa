with open('src/morphology/conjugation.rs', 'r') as f:
    content = f.read()

# wait, we found that analyze_verb doesn't actually call analyze_verb_all at all. It uses try_conjugation_pattern which uses match_verb_endings!
# So there's no Vec::new() allocated in analyze_verb.
