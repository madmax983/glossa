use glossa::codegen::{sanitize_name, to_rust_type, generate_type_tokens};
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

#[test]
fn test_sanitizer_edge_cases() {
    // Test empty string
    let empty = sanitize_name("");
    // Should contain "g_" and "_var_empty"
    assert_eq!(empty, "g__var_empty");
}

#[test]
fn test_generate_type_tokens_coverage() {
    // We need to test all branches of generate_type_tokens to ensure high code coverage

    // Number
    let t_num = generate_type_tokens(&GlossaType::Number);
    assert_eq!(t_num.to_string(), "i64");

    // String
    let t_str = generate_type_tokens(&GlossaType::String);
    assert_eq!(t_str.to_string(), "String");

    // Boolean
    let t_bool = generate_type_tokens(&GlossaType::Boolean);
    assert_eq!(t_bool.to_string(), "bool");

    // Unit
    let t_unit = generate_type_tokens(&GlossaType::Unit);
    assert_eq!(t_unit.to_string(), "()");

    // Unknown
    let t_unknown = generate_type_tokens(&GlossaType::Unknown);
    assert_eq!(t_unknown.to_string(), "_");

    // Function
    let t_fn = generate_type_tokens(&GlossaType::Function {
        params: vec![],
        returns: Box::new(GlossaType::Unit)
    });
    assert_eq!(t_fn.to_string(), "fn");

    // List
    let t_list = generate_type_tokens(&GlossaType::List(Box::new(GlossaType::Number)));
    assert_eq!(t_list.to_string(), "Vec < i64 >");

    // Set
    let t_set = generate_type_tokens(&GlossaType::Set(Box::new(GlossaType::Number)));
    assert_eq!(t_set.to_string(), "HashSet < i64 >");

    // Map
    let t_map = generate_type_tokens(&GlossaType::Map(
        Box::new(GlossaType::String),
        Box::new(GlossaType::Number)
    ));
    assert_eq!(t_map.to_string(), "HashMap < String , i64 >");

    // Option
    let t_opt = generate_type_tokens(&GlossaType::Option(Box::new(GlossaType::Number)));
    assert_eq!(t_opt.to_string(), "Option < i64 >");

    // Result
    let t_res = generate_type_tokens(&GlossaType::Result(
        Box::new(GlossaType::Number),
        Box::new(GlossaType::String)
    ));
    assert_eq!(t_res.to_string(), "Result < i64 , String >");

    // Struct
    let t_struct = generate_type_tokens(&GlossaType::Struct {
        name: SmolStr::new("test"),
        fields: vec![],
        gender: Gender::Neuter,
    });
    // Sanitize "test" -> "g_test", Capitalize -> "G_test"
    assert_eq!(t_struct.to_string(), "G_test");
}
