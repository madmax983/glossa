#![allow(missing_docs)]
//! Warden Exploit Test: Index Truncation
//!
//! This test demonstrates the vulnerability where large `i64` indices are truncated
//! when cast to `usize` on 32-bit platforms, potentially allowing out-of-bounds access.

#[test]
fn test_index_truncation_simulation() {
    // Simulate a 32-bit environment
    // Max u32 is 4,294,967,295

    // A large index that wraps to 1 in 32-bit
    // 2^32 + 1 = 4,294,967,297
    let large_index: i64 = 4_294_967_297;

    // The vulnerable pattern: `idx as usize` (simulated as cast to u32 then to usize)
    let truncated_index = (large_index as u32) as usize;

    // Demonstrate that truncation happens
    assert_eq!(truncated_index, 1);

    // If we had an array of size 2, this would access index 1 instead of panicking!
    let arr = [10, 20];
    assert_eq!(arr[truncated_index], 20); // Should have panicked!

    // The Fix: using try_from
    let safe_index = usize::try_from(large_index);

    if cfg!(target_pointer_width = "32") {
        // On actual 32-bit system, this should fail
        assert!(safe_index.is_err());
    } else {
        // On 64-bit system, it might succeed if memory allows, but try_from is still correct.
        // However, standard `as usize` on 64-bit preserves value, so no truncation there.
        // The point is that on 32-bit, `try_from` returns Err, while `as usize` returns junk.

        // Let's simulate the check we want to insert
        let check = u32::try_from(large_index); // Simulate checking against 32-bit limit
        assert!(check.is_err());
    }
}
