#![allow(missing_docs)]
use glossa::text::normalize_greek;
use std::time::Instant;

#[test]
fn test_dos_sigma_normalization() {
    let n = 20_000;
    let s: String = "Σ".repeat(n);

    let start = Instant::now();
    let _normalized = normalize_greek(&s);
    let duration = start.elapsed();

    println!("Time for {} sigmas: {:?}", n, duration);

    // Performance assertion (sanity check for DoS regression)
    // 20k sigmas should take well under 100ms on any modern machine if linear
    assert!(
        duration.as_millis() < 500,
        "Normalization took too long: {:?} (Should be linear)",
        duration
    );
}
