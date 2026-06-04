use glossa::tools::repl::run_repl_inner;
use std::io::Read;

struct InfiniteSpaces;
impl Read for InfiniteSpaces {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for b in buf.iter_mut() {
            *b = b' ';
        }
        Ok(buf.len())
    }
}

#[test]
#[ignore]
fn test_repl_infinite_exhaustion() {
    let input = InfiniteSpaces;
    let mut buf_input = std::io::BufReader::new(input);
    let mut output = Vec::new();
    let _ = run_repl_inner(&mut buf_input, &mut output);
}
