fn main() {
    let s = "«χαῖρε κόσμε» λέγε.";
    let words: Vec<String> = s
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect();
    println!("{:?}", words);
}
