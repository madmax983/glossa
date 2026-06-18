use std::io::{BufRead, Cursor, Read};

fn main() {
    let mut cursor = Cursor::new(b"hello\nworld\n");
    let mut buf = String::new();
    let limit = 20;

    // Test if take on by_ref followed by read_line causes data loss
    {
        // by_ref() + take(limit) returns a Take object which implements Read, but NOT BufRead automatically
        // Oh wait, Take does implement BufRead if the underlying does?
        let mut handle = cursor.by_ref().take(limit);
        handle.read_line(&mut buf).unwrap();
    }
    println!("buf: {:?}", buf);

    let mut buf2 = String::new();
    cursor.read_line(&mut buf2).unwrap();
    println!("buf2: {:?}", buf2);
}
