#![allow(missing_docs)]

use glossa::parser::parse;
use glossa::semantic::analyze_program;
use std::env;
use std::process::Command;

#[test]
fn test_binding_swap_undefined_panic() {
    // We are expecting a PANIC! If it doesn't panic, our task is done.
    // Wait, the memory says "ensure the bound variable is completely removed ... so that subsequent value extraction doesn't attempt to process the undefined name and panic."
    // If I FIX it, it WON'T panic. The test should check that it DOES panic BEFORE the fix, and DOES NOT panic AFTER the fix.
    // But since I am Havoc, I am supposed to write a test that PROVES it panics!
    // But I must also leave the test failing?
    // "Red Phase: Write tests that fail"
    // "Green Phase: Write the minimal amount of code to make those tests pass."
    // OK, so I write a test that expects NO PANIC (i.e. it passes if there's no panic).
    // And before the fix, it panics (so the test fails).
    // Wait, the issue says:
    // "If it works, you failed."
    // "Never write 'Happy Path' tests."

    // Ah, Havoc writes a test that *asserts the panic happens* (e.g. `#[should_panic]`), OR spawns a process and asserts `!status.success()`.
    // THEN, when the bug is fixed, the test fails because it *no longer panics*!
    // Wait... if I fix the bug, then `!status.success()` becomes false, so the test panics: "Process did not crash as expected!"
    // Yes! That's exactly what was happening! My fix was working perfectly, and the test was failing because my fix WORKED and prevented the crash!

    // I am NOT supposed to fix the bug if I am in STRICT CHAOS MODE.
    // "Never do: Fix the bug."
    // "If it crashes/panics/deadlocks: SUCCESS."

    if env::var("HAVOC_TRIGGER_CRASH_SWAP").is_ok() {
        let code = r#"
            // The object 'ψ' is NOT defined, so it will be the bound variable.
            // The subject 'ξ' is defined, so it will be the value to extract.
            // But wait, the original code left 'ψ' in `fixed.object`! Wait, no, `swapped.subject = Some(object.clone())`.
            // So `swapped.subject` is `ψ`. And `swapped.object` is `ξ`.
            // Then `extract_value` processes `swapped.object` (ξ) successfully.
            // Does it process `swapped.subject` (ψ)?
            // It only does if there's an `extract_subject_fallback`. But there IS NOT!
            // So it actually DOESN'T panic in `extract_value`.

            // Where DOES it panic?
            // "so that subsequent value extraction doesn't attempt to process the undefined name and panic."
            // Value extraction looks at `asm_stmt.unwraps`.
            // Let's use `unwraps`!
            // "ψ! λέγε." - no, we need `ξ ψ! ἔστω.`
            ξ 5 ἔστω.
            ξ ψ! ἔστω.
        "#;
        let ast = parse(code).unwrap();
        let _ = analyze_program(&ast).unwrap();
        return;
    }

    let exe = env::current_exe().unwrap();
    let status = Command::new(exe)
        .arg("test_binding_swap_undefined_panic")
        .arg("--exact")
        .arg("--test-threads=1")
        .env("HAVOC_TRIGGER_CRASH_SWAP", "1")
        .status()
        .unwrap();

    assert!(!status.success(), "Process did not crash as expected!");
}
