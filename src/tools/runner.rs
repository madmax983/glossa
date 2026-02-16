use crate::codegen::generate_rust_file;
use crate::parser::parse;
use crate::report::{CompilationReport, GlossaReport, ProgramStats};
use crate::semantic::{AnalyzedProgram, analyze_program};
use crate::tools::highlight::highlight;
use crossterm::{ExecutableCommand, cursor, style::Stylize, terminal};
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime};

/// Maximum source file size (1MB) to prevent memory exhaustion
const MAX_FILE_SIZE: u64 = 1024 * 1024;

fn analyze_source(source: &str) -> Result<AnalyzedProgram> {
    let ast = parse(source).map_err(|e| miette::miette!("{}", e))?;
    analyze_program(&ast).map_err(|e| miette::miette!("{}", e))
}

fn compile(source: &str) -> Result<String> {
    let analyzed = analyze_source(source)?;
    Ok(generate_rust_file(&analyzed))
}

/// Check file size to prevent DoS
fn check_file_size(input: &Path) -> Result<()> {
    let metadata = fs::metadata(input).into_diagnostic()?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(miette::miette!(
            "Ἀρχεῖον λίαν μέγα (File too large): {} > {} bytes",
            metadata.len(),
            MAX_FILE_SIZE
        ));
    }
    Ok(())
}

fn load_source(input: &Path) -> Result<String> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }
    check_file_size(input)?;

    let file = fs::File::open(input).into_diagnostic()?;
    let mut content = String::new();

    // Use take to limit the read, preventing OOM on infinite streams (e.g. /dev/zero)
    file.take(MAX_FILE_SIZE + 1)
        .read_to_string(&mut content)
        .into_diagnostic()?;

    if content.len() as u64 > MAX_FILE_SIZE {
        return Err(miette::miette!(
            "Ἀρχεῖον λίαν μέγα (File too large): > {} bytes",
            MAX_FILE_SIZE
        ));
    }

    Ok(content)
}

pub fn build_file(input: &Path, output: Option<&Path>) -> Result<()> {
    let status = Status::start("Μεταγλώττισις (Compiling)");
    let start = std::time::Instant::now();
    let source = load_source(input)?;
    let input_size = source.len() as u64;

    // Split compile to get stats
    let analyzed = analyze_source(&source)?;
    let rust_code = generate_rust_file(&analyzed);

    let output_path = output
        .map(|p| p.to_owned())
        .unwrap_or_else(|| input.with_extension("rs"));

    fs::write(&output_path, &rust_code).into_diagnostic()?;

    let output_size = fs::metadata(&output_path).into_diagnostic()?.len();
    let duration = start.elapsed();
    let stats = ProgramStats::new(&analyzed);

    status.success();

    let report = CompilationReport {
        input_path: input.to_path_buf(),
        output_path,
        input_size,
        output_size,
        duration,
        stats,
    };

    println!("{}", report);

    Ok(())
}

pub fn run_file(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }
    check_file_size(input)?;

    // Set up cache
    let cache = Cache::new();
    cache.init().into_diagnostic()?;

    let (cached_rs, cached_exe) = cache.get_paths(input);

    // Check if we can use cached binary
    if cache.is_valid(input, &cached_exe) && cached_exe.exists() {
        // Run cached binary directly
        let exit_status = Command::new(&cached_exe).status().into_diagnostic()?;

        if !exit_status.success() {
            std::process::exit(exit_status.code().unwrap_or(1));
        }
        return Ok(());
    }

    let mut status = Status::start("Μεταγλώττισις (Compiling)");

    // Compile source
    let source = load_source(input)?;

    let rust_code = match compile(&source) {
        Ok(code) => code,
        Err(e) => {
            status.error("Σφάλμα μεταγλωττίσεως");
            return Err(e);
        }
    };

    // Write Rust source to cache
    fs::write(&cached_rs, &rust_code).into_diagnostic()?;

    status.update("Οἰκοδόμησις (Building)");

    // Compile with rustc (hide output)
    let rustc_output = Command::new("rustc")
        .arg(&cached_rs)
        .arg("-o")
        .arg(&cached_exe)
        .arg("-O") // Optimize for speed
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .into_diagnostic()?;

    if !rustc_output.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_output.stderr);
        status.error("Σφάλμα κώδικος (Codegen Error)");
        return Err(miette::miette!("{}\n{}", "Rustc Error:".red(), stderr));
    }

    status.success();

    // Run the compiled program
    let exit_status = Command::new(&cached_exe).status().into_diagnostic()?;

    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }

    Ok(())
}

pub fn check_file(input: &Path) -> Result<()> {
    let source = load_source(input)?;

    let analyzed = analyze_source(&source)?;

    let filename = input
        .file_name()
        .unwrap_or(input.as_os_str())
        .to_string_lossy()
        .to_string();
    let report = GlossaReport::new(&analyzed, filename);

    println!("{}", report);

    Ok(())
}

pub fn highlight_file(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let highlighted = highlight(&source).map_err(|e| miette::miette!("{}", e))?;

    println!("{}", highlighted);

    Ok(())
}

// --- Internal Modules (Flattened) ---

/// Manages the build cache for compiled programs.
struct Cache {
    base_dir: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    /// Create a new Cache manager, resolving the cache directory.
    fn new() -> Self {
        let base_dir = dirs_next::cache_dir()
            .or_else(dirs_next::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".glossa")
            .join("cache");
        Self { base_dir }
    }

    /// Ensure the cache directory exists.
    fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_dir)
    }

    /// Generate a cache key from the source file path.
    fn key(&self, input: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
        let mut hasher = DefaultHasher::new();
        canonical.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Get the paths for the cached Rust source and executable.
    fn get_paths(&self, input: &Path) -> (PathBuf, PathBuf) {
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
    fn is_valid(&self, input: &Path, cached_exe: &Path) -> bool {
        let source_modified = fs::metadata(input)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let exe_modified = fs::metadata(cached_exe)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        exe_modified > source_modified
    }
}

/// Status indicator for long-running operations
struct Status {
    message: String,
    start: Instant,
    is_tty: bool,
    active: bool,
}

impl Status {
    /// Create a new status indicator
    fn start(message: impl Into<String>) -> Self {
        let is_tty = io::stderr().is_terminal();
        Self::new(message, is_tty)
    }

    /// Internal constructor for testing
    fn new(message: impl Into<String>, is_tty: bool) -> Self {
        let message = message.into();

        if is_tty {
            let mut stderr = io::stderr();
            // Hide cursor
            let _ = stderr.execute(cursor::Hide);
            // Print status
            eprint!("{} {}...", "⚡".yellow(), message.clone().bold());
            let _ = io::stderr().flush();
        } else {
            // For non-TTY, just print line
            eprintln!("{} {}...", "⚡".yellow(), message.clone().bold());
        }

        Self {
            message,
            start: Instant::now(),
            is_tty,
            active: true,
        }
    }

    /// Update the status message
    fn update(&mut self, message: impl Into<String>) {
        if !self.active {
            return;
        }

        let message = message.into();
        self.message = message.clone();

        if self.is_tty {
            let mut stderr = io::stderr();
            // Clear current line
            eprint!("\r");
            let _ = stderr.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
            // Print new status
            eprint!("{} {}...", "⚡".yellow(), message.bold());
            let _ = io::stderr().flush();
        } else {
            eprintln!("{} {}...", "⚡".yellow(), message.bold());
        }
    }

    /// Mark the operation as complete success
    fn success(mut self) {
        if !self.active {
            return;
        }

        let duration = self.start.elapsed();
        let time_str = format!("({:.2?})", duration).dim();

        if self.is_tty {
            let mut stderr = io::stderr();
            // Clear line
            eprint!("\r");
            let _ = stderr.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
            // Print success
            eprintln!(
                "{} {} {}",
                "✓".green(),
                self.message.as_str().bold(),
                time_str
            );
            // Show cursor
            let _ = stderr.execute(cursor::Show);
        } else {
            eprintln!(
                "{} {} {}",
                "✓".green(),
                self.message.as_str().bold(),
                time_str
            );
        }

        self.active = false;
    }

    /// Mark the operation as failed
    fn error(mut self, err: impl std::fmt::Display) {
        if !self.active {
            return;
        }

        if self.is_tty {
            let mut stderr = io::stderr();
            eprint!("\r");
            let _ = stderr.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
            eprintln!("{} {}", "✕".red(), self.message.as_str().bold());
            // Show cursor
            let _ = stderr.execute(cursor::Show);
        } else {
            eprintln!("{} {}", "✕".red(), self.message.as_str().bold());
        }

        eprintln!("{}", err);
        self.active = false;
    }
}

impl Drop for Status {
    fn drop(&mut self) {
        if self.active && self.is_tty {
            let _ = io::stderr().execute(cursor::Show);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_compile_hello() {
        let source = "«χαῖρε κόσμε» λέγε.";
        let result = compile(source);
        assert!(result.is_ok());
        let code = result.unwrap();
        // quote! generates `println !` with space
        assert!(code.contains("println"), "Expected println in: {}", code);
    }

    #[test]
    fn test_compile_binding() {
        let source = "ξ πέντε ἔστω.";
        let result = compile(source);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("let"));
    }

    #[test]
    fn test_compile_full_program() {
        let source = "ξ πέντε ἔστω. ξ λέγε.";
        let result = compile(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_size_check_internal() {
        // Create large file
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("large_internal.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            let data = vec![0u8; (MAX_FILE_SIZE + 1) as usize];
            f.write_all(&data).unwrap();
        }

        // Call check_file_size directly
        let result = check_file_size(&file_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Ἀρχεῖον λίαν μέγα")
        );
    }

    #[test]
    fn test_build_file_size_limit() {
        // Create large file
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("large_build.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            let data = vec![0u8; (MAX_FILE_SIZE + 1) as usize];
            f.write_all(&data).unwrap();
        }

        // Call build_file
        let result = build_file(&file_path, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Ἀρχεῖον λίαν μέγα")
        );
    }

    #[test]
    fn test_build_file_success() {
        // Create a temporary input file
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("test.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«test» λέγε.".as_bytes()).unwrap();
        }

        // Call build_file
        let result = build_file(&input_path, None);
        assert!(result.is_ok());

        // Verify output file exists
        let output_path = input_path.with_extension("rs");
        assert!(output_path.exists());

        // Output size is > 0
        let metadata = std::fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_run_file_size_limit() {
        // Create large file
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("large_run.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            let data = vec![0u8; (MAX_FILE_SIZE + 1) as usize];
            f.write_all(&data).unwrap();
        }

        // Call run_file (should fail at size check before running rustc)
        let result = run_file(&file_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Ἀρχεῖον λίαν μέγα")
        );
    }

    #[test]
    fn test_run_file_success() {
        // 1. Create a temporary Glossa file
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("run_test.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«test» λέγε.".as_bytes()).unwrap();
        }

        // 2. Run it
        // This exercises: run_file -> check_file_size -> cache_dir -> cache_key -> compile -> rustc -> execution
        let result = run_file(&input_path);

        // Note: this test requires `rustc` to be in the path, which is true for `cargo test`.
        assert!(result.is_ok());

        // 3. Verify cache exists
        let cache = Cache::new();
        // We can't easily check internal dir existence without exposing it,
        // but we can check if the exe exists via get_paths
        let (_, cached_exe) = cache.get_paths(&input_path);
        assert!(cached_exe.exists());

        // 4. Run again to test cache hit path
        let result_cached = run_file(&input_path);
        assert!(result_cached.is_ok());
    }

    #[test]
    fn test_run_compile_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("error.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("invalid syntax".as_bytes()).unwrap();
        }

        let result = run_file(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Σφάλμα"));
    }

    #[test]
    fn test_run_rustc_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("rustc_error.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            // This is valid Glossa but invalid Rust (redefining String)
            // Memory says: εἶδος String ὁρίζειν...
            f.write_all("εἶδος String ὁρίζειν { }. τέλος.".as_bytes())
                .unwrap();
        }

        let result = run_file(&input_path);
        assert!(result.is_err());
        // Verify it hits the rustc error path
        assert!(result.unwrap_err().to_string().contains("Rustc Error"));
    }

    #[test]
    fn test_check_file_valid() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("check.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ πέντε ἔστω.".as_bytes()).unwrap();
        }

        let result = check_file(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_highlight_file_valid() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("highlight.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ πέντε ἔστω.".as_bytes()).unwrap();
        }

        // We can't easily capture stdout here without a lot of plumbing,
        // but we can ensure it doesn't error.
        let result = highlight_file(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_status_tty_success() {
        // TTY branches
        let status = Status::new("Testing TTY", true);
        status.success();
    }

    #[test]
    fn test_status_tty_update() {
        let mut status = Status::new("Testing TTY Update", true);
        status.update("Updated");
        status.success();
    }

    #[test]
    fn test_status_tty_error() {
        let status = Status::new("Testing TTY Error", true);
        status.error("Something went wrong");
    }

    #[test]
    fn test_status_no_tty_success() {
        // Non-TTY branches
        let status = Status::new("Testing No-TTY", false);
        status.success();
    }

    #[test]
    fn test_status_no_tty_update() {
        let mut status = Status::new("Testing No-TTY Update", false);
        status.update("Updated");
        status.success();
    }

    #[test]
    fn test_status_no_tty_error() {
        let status = Status::new("Testing No-TTY Error", false);
        status.error("Something went wrong");
    }

    #[test]
    fn test_status_drop() {
        {
            let _status = Status::new("Testing Drop", true);
            // Should execute Drop (show cursor)
        }
    }

    #[test]
    fn test_cache_key_generation() {
        let cache = Cache::new();
        let path = Path::new("test.gl");
        let key1 = cache.key(path);
        let key2 = cache.key(path);
        // Keys should be deterministic
        assert_eq!(key1, key2);
        // SHA-256 hash length (hex)
        assert_eq!(key1.len(), 16);
    }

    #[test]
    fn test_cache_is_valid() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("src.gl");
        let output = dir.path().join("out.exe");

        // Create input file
        {
            let mut f = std::fs::File::create(&input).unwrap();
            f.write_all(b"content").unwrap();
        }

        // Wait to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Create output file (newer)
        {
            let mut f = std::fs::File::create(&output).unwrap();
            f.write_all(b"exe").unwrap();
        }

        let cache = Cache::new();
        // Output is newer than input -> valid
        assert!(
            cache.is_valid(&input, &output),
            "Cache should be valid when output is newer"
        );

        // Wait again
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Touch input to make it newer
        {
            let mut f = std::fs::File::create(&input).unwrap();
            f.write_all(b"new content").unwrap();
        }

        // Input is newer than output -> invalid
        assert!(
            !cache.is_valid(&input, &output),
            "Cache should be invalid when input is newer"
        );
    }
}
