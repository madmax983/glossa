use glossa::tools::Cache;
use std::path::Path;

#[test]
fn test_cache_key_determinism() {
    let cache = Cache::new();
    let path = Path::new("src/main.rs");

    // Key generation should be deterministic
    let key1 = cache.key(path);
    let key2 = cache.key(path);

    assert_eq!(key1, key2, "Cache key must be deterministic");
}

#[test]
fn test_cache_key_format() {
    let cache = Cache::new();
    let path = Path::new("src/main.rs");
    let key = cache.key(path);

    // SHA-256 hex string is 64 chars
    assert_eq!(
        key.len(),
        64,
        "Cache key must be 64 characters long (SHA-256 hex)"
    );

    // Check if it's hex
    assert!(
        key.chars().all(|c: char| c.is_ascii_hexdigit()),
        "Cache key must be hex string"
    );
}

#[test]
fn test_cache_key_uniqueness() {
    let cache = Cache::new();
    // Use paths that resolve to different locations
    let path1 = Path::new("src/main.rs");
    let path2 = Path::new("src/lib.rs");

    let key1 = cache.key(path1);
    let key2 = cache.key(path2);

    assert_ne!(key1, key2, "Different paths must produce different keys");
}

#[test]
fn test_cache_key_canonicalization() {
    // If the file exists, canonicalization should work and produce the same key for relative vs absolute
    // This depends on file system state, so we use existing files.
    let cache = Cache::new();
    let path = Path::new("src/main.rs");

    if path.exists() {
        let abs_path = path.canonicalize().unwrap();
        let key_rel = cache.key(path);
        let key_abs = cache.key(&abs_path);

        assert_eq!(
            key_rel, key_abs,
            "Relative and absolute paths should hash to same key"
        );
    }
}
