use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

use crate::codegen::generate_rust_file;
use crate::errors::GlossaError;
use crate::parser::parse;
use crate::report::{CompilationReport, GlossaReport, ProgramStats};
use crate::semantic::analyze_program;
use crate::tools::highlight::highlight;

/// Maximum source file size (1MB) to prevent memory exhaustion
const MAX_FILE_SIZE: u64 = 1024 * 1024;

fn compile(source: &str) -> std::result::Result<String, GlossaError> {
    let ast = parse(source)?;
    let analyzed = analyze_program(&ast)?;
    Ok(generate_rust_file(&analyzed))
}

/// Get the cache directory for compiled programs
fn cache_dir() -> PathBuf {
    let base = dirs_next::cache_dir()
        .or_else(dirs_next::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".glossa").join("cache")
}

/// Generate a cache key from source file path
fn cache_key(input: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Check if cached binary is still valid (source not modified since compile)
fn cache_valid(input: &Path, cached_exe: &Path) -> bool {
    let source_modified = fs::metadata(input)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let exe_modified = fs::metadata(cached_exe)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    exe_modified > source_modified
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

pub fn build_file(input: &Path, output: Option<&Path>) -> Result<()> {
    check_file_size(input)?;

    let start = std::time::Instant::now();
    let source = fs::read_to_string(input).into_diagnostic()?;
    let input_size = source.len() as u64;

    // Split compile to get stats
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let analyzed = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;
    let rust_code = generate_rust_file(&analyzed);

    let output_path = output
        .map(|p| p.to_owned())
        .unwrap_or_else(|| input.with_extension("rs"));

    fs::write(&output_path, &rust_code).into_diagnostic()?;

    let output_size = fs::metadata(&output_path).into_diagnostic()?.len();
    let duration = start.elapsed();
    let stats = ProgramStats::new(&analyzed);

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
    // Validate file exists
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    check_file_size(input)?;

    // Set up cache directory
    let cache = cache_dir();
    fs::create_dir_all(&cache).into_diagnostic()?;

    let key = cache_key(input);
    let cached_rs = cache.join(format!("{}.rs", key));
    let cached_exe = cache.join(format!(
        "{}{}",
        key,
        if cfg!(windows) { ".exe" } else { "" }
    ));

    // Check if we can use cached binary
    if cache_valid(input, &cached_exe) && cached_exe.exists() {
        // Run cached binary directly
        let status = Command::new(&cached_exe).status().into_diagnostic()?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        return Ok(());
    }

    // Compile source
    let source = fs::read_to_string(input).into_diagnostic()?;
    let rust_code = compile(&source).map_err(|e| miette::miette!("{}", e))?;

    // Write Rust source to cache
    fs::write(&cached_rs, &rust_code).into_diagnostic()?;

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
        // Show rustc errors only on failure
        let stderr = String::from_utf8_lossy(&rustc_output.stderr);
        return Err(miette::miette!("Σφάλμα μεταγλωττίσεως:\n{}", stderr));
    }

    // Run the compiled program
    let status = Command::new(&cached_exe).status().into_diagnostic()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

pub fn check_file(input: &Path) -> Result<()> {
    check_file_size(input)?;

    let source = fs::read_to_string(input).into_diagnostic()?;

    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let analyzed = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

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
    check_file_size(input)?;

    let source = fs::read_to_string(input).into_diagnostic()?;
    let highlighted = highlight(&source).map_err(|e| miette::miette!("{}", e))?;

    println!("{}", highlighted);

    Ok(())
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
}
