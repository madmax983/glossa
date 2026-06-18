use std::io::{BufRead, Cursor, Read};

fn read_line_bounded<R: BufRead>(reader: &mut R, buf: &mut String, limit: usize) -> std::io::Result<usize> {
    reader.by_ref().take(limit as u64).read_line(buf)
}

fn main() {
    let mut cursor = Cursor::new(b"hello\nworld\n");
    let mut buf = String::new();
    let limit = 20;
    read_line_bounded(&mut cursor, &mut buf, limit).unwrap();
    println!("buf: {:?}", buf);
}
