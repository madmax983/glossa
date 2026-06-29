with open('src/morphology/conjugation.rs', 'r') as f:
    content = f.read()

search_str = """pub fn analyze_verb(word: &str) -> Option<MorphAnalysis> {
    let word_string = normalize_greek(word);
    let word = word_string.as_str();"""

replace_str = """pub fn analyze_verb(word: &str) -> Option<MorphAnalysis> {
    let word_string = normalize_greek(word);
    let word = word_string.as_str();"""

# I need to find the `let mut analyses = analyze_verb_all(word);` call instead of just the start of analyze_verb.
