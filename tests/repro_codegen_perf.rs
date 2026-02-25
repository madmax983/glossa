use glossa::codegen::{sanitize_name, to_rust_type};
use glossa::semantic::GlossaType;
use glossa::morphology::Gender;
use smol_str::SmolStr;

#[test]
fn test_allocations() {
    // This is just a functional test to ensure behavior is preserved.
    // Measuring allocations in a test is hard without custom allocator or specific tools.
    // But we can verify correctness.

    let name = "χρηστης";
    let sanitized = sanitize_name(name);
    // Sanitize adds "g_" prefix and hex encodes Greek chars
    // x (chi) -> _u3c7_
    // r (rho) -> _u3c1_
    // h (eta) -> _u3b7_
    // s (sigma) -> _u3c3_
    // t (tau) -> _u3c4_
    // h (eta) -> _u3b7_
    // s (sigma final) -> _u3c2_
    assert_eq!(sanitized, "g__u3c7__u3c1__u3b7__u3c3__u3c4__u3b7__u3c2_");

    let ty = GlossaType::Struct {
        name: SmolStr::new(name),
        // Adding dummy fields just to construct the enum variant
        fields: vec![],
        gender: Gender::Masculine,
    };

    // to_rust_type capitalizes the sanitized name
    // g_... -> G_...
    let rust_type = to_rust_type(&ty);
    assert_eq!(rust_type, "G__u3c7__u3c1__u3b7__u3c3__u3c4__u3b7__u3c2_");
}
