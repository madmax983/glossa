use glossa::codegen::generate_rust;
use glossa::semantic::{AnalyzedProgram, AnalyzedStatement, GlossaType, Scope};
use smol_str::SmolStr;

#[test]
fn test_complex_types_panic_and_import() {
    let program = AnalyzedProgram {
        statements: vec![
            // Type Definition: struct Complex { map: HashMap<String, i64>, list: Vec<i64> }
            AnalyzedStatement::TypeDefinition {
                name: SmolStr::new("Complex"),
                fields: vec![
                    (
                        SmolStr::new("map"),
                        GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)),
                    ),
                    (
                        SmolStr::new("list"),
                        GlossaType::List(Box::new(GlossaType::Number)),
                    ),
                ],
            },
            // Function Definition: fn process(opt: Option<i64>) -> Result<String, String>
            AnalyzedStatement::FunctionDef {
                name: SmolStr::new("process"),
                params: vec![(
                    SmolStr::new("opt"),
                    Some(GlossaType::Option(Box::new(GlossaType::Number))),
                )],
                body: vec![],
                return_type: Some(GlossaType::Result(
                    Box::new(GlossaType::String),
                    Box::new(GlossaType::String),
                )),
            },
        ],
        scope: Scope::default(),
    };

    let code = generate_rust(&program);
    println!("{}", code);

    // Normalize whitespace for robust checking
    let normalized = code.replace(" ", "").replace("\n", "");

    // Check for correct imports
    // "use std :: collections :: { HashMap , HashSet } ;" -> "usestd::collections::{HashMap,HashSet};"
    assert!(
        normalized.contains("usestd::collections::{HashMap,HashSet};"),
        "Missing imports for Map/Set"
    );

    // Check for correct type generation
    // "HashMap < String , i64 >" -> "HashMap<String,i64>"
    assert!(
        normalized.contains("HashMap<String,i64>"),
        "HashMap not generated correctly"
    );
    assert!(
        normalized.contains("Vec<i64>"),
        "Vec not generated correctly"
    );
}
