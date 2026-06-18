use std::io::{BufRead, Cursor, Read};

fn main() {
    let mut cursor = Cursor::new(b"hello\nworld\n");
    let mut buf = String::new();
    let mut reader = cursor.by_ref();
    let limit = 20;
    (&mut *reader).take(limit as u64).read_line(&mut buf).unwrap();
    println!("buf: {:?}", buf);
}
