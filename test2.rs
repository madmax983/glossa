fn words_iter<'a>(expr: &'a Expr) -> Box<dyn Iterator<Item = &'a Word> + 'a> {
    match expr {
        Expr::Word(word) => Box::new(std::iter::once(word)),
        Expr::Phrase(terms) => Box::new(terms.iter().flat_map(|t| words_iter(t))),
        _ => Box::new(std::iter::empty()),
    }
}
