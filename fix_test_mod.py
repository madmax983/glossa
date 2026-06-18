import re

with open('src/tools/mod.rs', 'r') as f:
    mod = f.read()

# I also need to make sure the tests I supposedly added in the trace but maybe failed to are present
if "test_read_line_bounded" not in mod:
    tests = """
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_line_bounded_normal() {
        let mut cursor = Cursor::new(b"hello\\nworld");
        let mut buf = String::new();
        let bytes = read_line_bounded(&mut cursor, &mut buf, 100).unwrap();
        assert_eq!(bytes, 6);
        assert_eq!(buf, "hello\\n");
    }

    #[test]
    fn test_read_line_bounded_limit() {
        let mut cursor = Cursor::new(b"hello\\nworld");
        let mut buf = String::new();
        // Limit is 3. Only "hel" should be read
        let bytes = read_line_bounded(&mut cursor, &mut buf, 3).unwrap();
        assert_eq!(bytes, 3);
        assert_eq!(buf, "hel");
    }

    #[test]
    fn test_read_line_bounded_eof() {
        let mut cursor = Cursor::new(b"hello");
        let mut buf = String::new();
        let bytes = read_line_bounded(&mut cursor, &mut buf, 100).unwrap();
        assert_eq!(bytes, 5);
        assert_eq!(buf, "hello");
    }

    #[test]
    fn test_warden_exploit_infinite_stream() {
        // Exploit attempt: simulate /dev/zero
        // A cursor that repeats 0 infinitely
        struct DevZero;
        impl std::io::Read for DevZero {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                for b in buf.iter_mut() {
                    *b = 0;
                }
                Ok(buf.len())
            }
        }
        impl std::io::BufRead for DevZero {
            fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
                static ZEROS: [u8; 1024] = [0; 1024];
                Ok(&ZEROS)
            }
            fn consume(&mut self, _amt: usize) {}
        }

        let mut zero = DevZero;
        let mut buf = String::new();
        let limit = 10_000;

        let bytes = read_line_bounded(&mut zero, &mut buf, limit).unwrap();
        assert_eq!(bytes, limit);
        assert_eq!(buf.len(), limit);
        // It successfully stops and does not OOM
    }
}
"""
    mod += tests
    with open('src/tools/mod.rs', 'w') as f:
        f.write(mod)
