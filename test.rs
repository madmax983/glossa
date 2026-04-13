fn collect_words_iter<'a>(expr: &'a Expr) -> Box<dyn Iterator<Item = &'a Word> + 'a> {
    match expr {
        Expr::Word(word) => Box::new(std::iter::once(word)),
        Expr::Phrase(terms) => Box::new(terms.iter().flat_map(|term| collect_words_iter(term))),
        _ => Box::new(std::iter::empty()),
    }
}
