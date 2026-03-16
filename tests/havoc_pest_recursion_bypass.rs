use std::env;
use std::process::Command;

/// 👺 Havoc: Parser Recursion Depth Scanner Bypass
///
/// The `check_recursion_depth` function performs a fast linear byte scan
/// to count nesting depth, preventing `pest` from causing a Stack Overflow.
///
/// It specifically checks for the exact UTF-8 bytes of `δοκιμή` (test declaration).
///
/// `if b == 0xCE && source[i..].starts_with("δοκιμή")`
///
/// However, `grammar.pest` accepts both `δοκιμή` and `δοκιμη` (without accent).
///
/// ```pest
/// dokime_keyword = @{ ("δοκιμή" | "δοκιμη") ~ !GREEK_CHAR }
/// ```
///
/// By using the unaccented `δοκιμη`, we completely bypass the fast byte scan,
/// allowing us to feed an infinitely nested structure directly into the `pest`
/// PEG parser, overflowing its stack and crashing the process.
#[test]
fn havoc_crash_pest_recursion_bypass() {
    // If we are running in the subprocess, execute the crash vector
    if env::var("HAVOC_DETONATE_PEST_BYPASS").is_ok() {
        use glossa::parser::parse;
        let mut source = String::new();
        // The limit is 250. We go to 10,000 to guarantee a stack overflow in pest.
        let depth = 10_000;

        // Use the UNACCENTED keyword to bypass check_recursion_depth
        for i in 0..depth {
            source.push_str(&format!("δοκιμη «{}» ", i));
        }

        source.push_str("1 λέγε. ");

        for _ in 0..depth {
            source.push_str("τελος. "); // Also use unaccented end
        }

        // 💥 DETONATE
        // This will bypass `check_recursion_depth` and go straight to `pest`.
        // Pest will recursively descend into `test_declaration` 10,000 times,
        // exhausting the thread stack and crashing.
        println!("Feeding deeply nested unaccented test declarations to parser...");
        let _result = parse(&source);

        // This line will never be reached!
        println!("Survived? Impossible.");
        std::process::exit(0);
    }

    // Otherwise, we are the test runner orchestrator.
    // Spawn a subprocess to run this exact test, and verify it CRASHES.
    let exe = env::current_exe().expect("Failed to get current executable");

    let status = Command::new(exe)
        .env("HAVOC_DETONATE_PEST_BYPASS", "1")
        .arg("--nocapture")
        .arg("havoc_crash_pest_recursion_bypass")
        .status()
        .expect("Failed to spawn subprocess");

    // The test SUCCEEDS if the subprocess FAILED (crashed via SIGSEGV/Stack Overflow)
    assert!(
        !status.success(),
        "Bug fixed? The subprocess should have crashed with a stack overflow!"
    );
}
