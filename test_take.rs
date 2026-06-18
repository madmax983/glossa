use std::io::{BufRead, Cursor, Read};

fn main() {
    let mut cursor = Cursor::new(b"hello world\n12345");
    let mut buf = String::new();
    cursor.by_ref().take(5).read_line(&mut buf).unwrap();
    println!("buf: {}", buf);

    let mut buf2 = String::new();
    cursor.read_line(&mut buf2).unwrap();
    println!("buf2: {}", buf2);
}
