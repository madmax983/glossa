fn main() {
    let x = Some(1);
    let y = Some(2);
    if let Some(a) = x && let Some(b) = y {
        println!("{}", a + b);
    }
}
