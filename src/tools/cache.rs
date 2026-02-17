use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Manages the build cache for compiled programs.
pub struct Cache {
    base_dir: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    /// Create a new Cache manager, resolving the cache directory.
    pub fn new() -> Self {
        let base_dir = dirs_next::cache_dir()
            .or_else(dirs_next::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".glossa")
            .join("cache");
        Self { base_dir }
    }

    /// Ensure the cache directory exists.
    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_dir)
    }

    /// Generate a cache key from the source file path.
    pub fn key(&self, input: &Path) -> String {
        let canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
        // Convert path to bytes. On unix this is OsStr bytes, on Windows it's UTF-8 if valid or WTF-8.
        // We use to_string_lossy() to get a consistent string representation for hashing.
        let path_str = canonical.to_string_lossy();

        let hash = fnv1a_64(path_str.as_bytes());
        format!("{:016x}", hash)
    }

    /// Get the paths for the cached Rust source and executable.
    pub fn get_paths(&self, input: &Path) -> (PathBuf, PathBuf) {
        let key = self.key(input);
        let cached_rs = self.base_dir.join(format!("{}.rs", key));
        let cached_exe = self.base_dir.join(format!(
            "{}{}",
            key,
            if cfg!(windows) { ".exe" } else { "" }
        ));
        (cached_rs, cached_exe)
    }

    /// Check if the cached binary is still valid (source not modified since compile).
    pub fn is_valid(&self, input: &Path, cached_exe: &Path) -> bool {
        let source_modified = fs::metadata(input)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let exe_modified = fs::metadata(cached_exe)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        exe_modified > source_modified
    }
}

const fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3); // Correct FNV prime
        i += 1;
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a_64_empty() {
        assert_eq!(fnv1a_64(b""), 0xcbf29ce484222325);
    }

    #[test]
    fn test_fnv1a_64_known_values() {
        // Known FNV-1a 64-bit hash values
        // "a"
        assert_eq!(fnv1a_64(b"a"), 0xaf63dc4c8601ec8c);
        // "foobar"
        assert_eq!(fnv1a_64(b"foobar"), 0x85944171f73967e8);
    }
}
