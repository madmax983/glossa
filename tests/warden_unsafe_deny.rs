use glossa::codegen::generate_rust_file;
use glossa::semantic::{AnalyzedProgram, Scope};

#[test]
fn test_warden_unsafe_deny() {
    let program = AnalyzedProgram {
        statements: vec![],
        scope: Scope::new(),
    };
    let code = generate_rust_file(&program);
    assert!(
        code.contains("#![deny(unsafe_code)]"),
        "Generated Rust files must deny unsafe code"
    );
}
