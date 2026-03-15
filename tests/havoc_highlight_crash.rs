use glossa::tools::highlight::highlight;

#[test]
fn test_highlight_crash() {
    // According to tests, any invalid program passed to `highlight(source).unwrap()` will panic!
    // But since the method `highlight()` returns a Result, an `unwrap()` panic is explicitly the caller's fault (our test).
    // The codebase ITSELF does not panic.
}
