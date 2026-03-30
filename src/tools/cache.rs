//! The Cache System - "The Vault"
//!
//! This module implements incremental compilation for ΓΛΩΣΣΑ.
//!
//! # The Vault Philosophy
//!
//! Compiling code takes time. To be fast, we must avoid doing work we've already done.
//! "The Vault" stores the results of previous compilations and retrieves them
//! if the source code hasn't changed.
//!
//! # How it works
//!
//! 1. **Key Generation**: A unique fingerprint (SHA-256 hash) is generated from the
//!    canonical path of the source file. This ensures that `src/main.gl` and `./main.gl`
//!    map to the same cache entry.
//! 2. **Storage**: The compiler stores two artifacts in the cache directory (`~/.glossa/cache`):
//!    - `{hash}.rs`: The generated Rust source code.
//!    - `{hash}` (or `{hash}.exe`): The compiled binary.
//! 3. **Validation**: Before running, we check if the cached binary is newer than the source file.
//!    If it is, we skip compilation and run the cached binary directly (The "Hot Path").

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Manages the build cache for compiled programs.
///
/// Handles path resolution, key generation, and validity checking for cached binaries.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::Cache;
/// use std::path::Path;
///
/// let cache = Cache::new();
/// // In a real scenario, you'd call cache.init().unwrap() to ensure the directory exists
/// let (rs_path, exe_path) = cache.get_paths(Path::new("my_program.gl"));
/// ```
pub struct Cache {
    base_dir: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    /// Create a new Cache manager
    ///
    /// Resolves the cache directory to the system standard (e.g. `~/.cache` or `%LOCALAPPDATA%`)
    /// appended with `.glossa/cache`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::Cache;
    /// let cache = Cache::new();
    /// ```
    pub fn new() -> Self {
        let base_dir = dirs_next::cache_dir()
            .or_else(dirs_next::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".glossa")
            .join("cache");
        Self { base_dir }
    }

    /// Ensure the cache directory exists
    ///
    /// Creates the directory structure if it doesn't already exist.
    /// Should be called before writing any files.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use glossa::tools::Cache;
    /// let cache = Cache::new();
    /// cache.init().unwrap(); // Creates ~/.cache/.glossa/cache
    /// ```
    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_dir)
    }

    /// Generate a unique cache key for a source file
    ///
    /// The key is a SHA-256 hash of the **canonical path** of the input file.
    /// This ensures that the same file always maps to the same cache entry,
    /// regardless of how it was referenced (relative vs absolute path).
    ///
    /// # Why hash the path?
    ///
    /// We hash the path instead of the content for speed. The cache key serves to identify
    /// *which* file we are talking about, while [`is_valid`](Cache::is_valid) checks
    /// *if* it has changed.
    ///
    /// This separation allows:
    /// 1. Fast key generation (O(1) path operation vs O(N) file read).
    /// 2. Simple dependency tracking (path is stable identifier).
    ///
    /// The content change detection relies on filesystem timestamps, which is standard practice
    /// for build tools (like `make` or `cargo`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::Cache;
    /// use std::path::Path;
    ///
    /// let cache = Cache::new();
    /// let key = cache.key(Path::new("hero.gl"));
    /// assert_eq!(key.len(), 64); // SHA-256 hex string
    /// ```
    pub fn key(&self, input: &Path) -> String {
        use sha2::{Digest, Sha256};

        let canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
        // Convert path to bytes. On unix this is OsStr bytes, on Windows it's UTF-8 if valid or WTF-8.
        // We use to_string_lossy() to get a consistent string representation for hashing.
        // Ideally we'd use OsStr bytes directly but that's platform specific (OsStrExt).
        // For cross-platform consistency in this context (local cache), stringified path is fine.
        let path_str = canonical.to_string_lossy();

        let mut hasher = Sha256::new();
        hasher.update(path_str.as_bytes());
        let result = hasher.finalize();

        hex::encode(result)
    }

    /// Get the paths for the cached artifacts
    ///
    /// Returns a tuple of:
    /// 1. Path to the generated Rust source (`{hash}.rs`)
    /// 2. Path to the compiled executable (`{hash}` or `{hash}.exe`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::Cache;
    /// use std::path::Path;
    ///
    /// let cache = Cache::new();
    /// let (rs_file, exe_file) = cache.get_paths(Path::new("hero.gl"));
    /// assert!(rs_file.to_string_lossy().ends_with(".rs"));
    /// ```
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

    /// Check if the cached binary is still valid
    ///
    /// Returns `true` if the cached executable exists and is newer than the source file.
    /// This implements the standard "timestamp-based" incremental build logic.
    ///
    /// # Logic
    ///
    /// The cache is valid if and only if the cached executable exists AND
    /// is newer than the source file.
    ///
    /// ```text
    /// is_valid = exists(exe) && mtime(exe) > mtime(source)
    /// ```
    ///
    /// If the source file has been modified *after* the executable was built,
    /// `mtime(source)` will be greater, making the condition false (invalid).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use glossa::tools::Cache;
    /// use std::path::Path;
    ///
    /// let cache = Cache::new();
    /// let source_path = Path::new("main.gl");
    /// let (_, exe_path) = cache.get_paths(source_path);
    ///
    /// // If main.gl was modified recently, this returns false
    /// let is_hot = cache.is_valid(source_path, &exe_path);
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_default() {
        let cache = Cache::default();
        // Just assert it initializes without panicking.
        assert!(cache.base_dir.ends_with("cache"));
    }
}
