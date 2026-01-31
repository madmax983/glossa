fn main() {
    let s = "λύω";
    // We don't have access to internal lib here easily without cargo run,
    // but we can assume normalization removes accents based on "monotonic lowercase".
    // Actually, "monotonic" usually keeps one accent. "NFD normalization" splits chars.
    // Let's assume standard behavior: remove diacritics for analysis.
    // Or just check what the codebase does.
}
